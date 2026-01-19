use crate::dao;
use crate::dao::ham_club::{HamClub, NewHamClub};
use crate::dao::repeater_port::{NewRepeaterPort, RepeaterPort};
use crate::dao::repeater_service::{NewRepeaterService, RepeaterServiceKind};
use crate::dao::repeater_service_aprs::{AprsMode, NewRepeaterServiceAprs};
use crate::dao::repeater_service_c4fm::NewRepeaterServiceC4fm;
use crate::dao::repeater_service_dmr::NewRepeaterServiceDmr;
use crate::dao::repeater_service_dstar::{DstarMode, NewRepeaterServiceDstar};
use crate::dao::repeater_service_fm::{FmBandwidth, NewRepeaterServiceFm, ToneKind};
use crate::dao::repeater_site::{NewRepeaterSite, RepeaterSite};
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use diesel::QueryResult;
use diesel_async::AsyncPgConnection;

async fn ham_club(c: &mut AsyncPgConnection, call_sign: impl Into<String>) -> QueryResult<HamClub> {
    dao::ham_club::insert(c, NewHamClub::new(call_sign.into())).await
}

async fn repeater(
    c: &mut AsyncPgConnection,
    club: &Option<HamClub>,
    call_sign: impl Into<String>,
) -> QueryResult<RepeaterSystem> {
    let mut r = NewRepeaterSystem::new(call_sign);
    if let Some(club) = club {
        r = r.ham_club_id(club.id);
    }

    dao::repeater_system::insert(c, r).await
}

async fn repeater_site(
    c: &mut AsyncPgConnection,
    address: impl Into<String>,
) -> QueryResult<RepeaterSite> {
    dao::repeater_site::insert(c, NewRepeaterSite::address(address)).await
}

async fn create_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    label: impl Into<String>,
    tx_hz: i64,
    rx_hz: i64,
) -> QueryResult<RepeaterPort> {
    let port = NewRepeaterPort {
        repeater_id,
        label: label.into(),
        rx_hz,
        tx_hz,
        note: None,
    };

    dao::repeater_port::insert(c, port).await
}

async fn fm_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
    bandwidth: FmBandwidth,
    subtone: Option<f32>,
) -> QueryResult<()> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
            kind: RepeaterServiceKind::Fm,
            enabled: true,
        },
    )
    .await?;
    let mut fm = NewRepeaterServiceFm {
        service_id: service.id,
        bandwidth,
        access_tone_kind: ToneKind::None,
        access_ctcss_hz: None,
        access_dcs_code: None,
        tx_tone_kind: ToneKind::None,
        tx_ctcss_hz: None,
        tx_dcs_code: None,
    };

    if let Some(subtone) = subtone {
        fm = NewRepeaterServiceFm {
            access_tone_kind: ToneKind::CTCSS,
            access_ctcss_hz: Some(subtone),
            tx_tone_kind: ToneKind::CTCSS,
            tx_ctcss_hz: Some(subtone),
            ..fm
        }
    }

    dao::repeater_service_fm::insert(c, fm).await?;

    Ok(())
}

async fn dstar_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
) -> QueryResult<()> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
            kind: RepeaterServiceKind::Dstar,
            enabled: true,
        },
    )
    .await?;

    dao::repeater_service_dstar::insert(
        c,
        NewRepeaterServiceDstar {
            service_id: service.id,
            mode: DstarMode::Dv,
            gateway_call_sign: None,
            reflector: None,
        },
    )
    .await?;

    Ok(())
}

async fn dmr_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
    dmr_id: Option<i64>,
) -> QueryResult<()> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
            kind: RepeaterServiceKind::Dmr,
            enabled: true,
        },
    )
    .await?;

    dao::repeater_service_dmr::insert(
        c,
        NewRepeaterServiceDmr {
            service_id: service.id,
            color_code: None,
            dmr_repeater_id: dmr_id,
            network: None,
        },
    )
    .await?;

    Ok(())
}

async fn c4fm_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
) -> QueryResult<()> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
            kind: RepeaterServiceKind::C4fm,
            enabled: true,
        },
    )
    .await?;

    dao::repeater_service_c4fm::insert(
        c,
        NewRepeaterServiceC4fm {
            service_id: service.id,
            wires_x_node_id: None,
            room: None,
        },
    )
    .await?;

    Ok(())
}

pub async fn narrow_fm(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
    subtone: Option<f32>,
) -> QueryResult<RepeaterPort> {
    let port = create_port(c, r.id, label, tx_frequency, tx_frequency + offset).await?;
    fm_service_on_port(c, r.id, port.id, FmBandwidth::Narrow, subtone).await?;

    Ok(port)
}

pub async fn dstar(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
) -> QueryResult<RepeaterPort> {
    let port = create_port(c, r.id, label, tx_frequency, tx_frequency + offset).await?;
    dstar_service_on_port(c, r.id, port.id).await?;

    Ok(port)
}

pub async fn igate(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    frequency: i64,
) -> QueryResult<RepeaterPort> {
    let port = NewRepeaterPort {
        repeater_id: r.id,
        label: label.into(),
        rx_hz: frequency,
        tx_hz: frequency,
        note: None,
    };

    let port = dao::repeater_port::insert(c, port).await?;
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id: r.id,
            port_id: Some(port.id),
            kind: RepeaterServiceKind::Aprs,
            enabled: true,
        },
    )
    .await?;
    dao::repeater_service_aprs::insert(
        c,
        NewRepeaterServiceAprs {
            service_id: service.id,
            mode: AprsMode::Igate,
            path: None,
        },
    )
    .await?;

    Ok(port)
}

pub async fn digipeater(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    frequency: i64,
) -> QueryResult<RepeaterPort> {
    let port = NewRepeaterPort {
        repeater_id: r.id,
        label: label.into(),
        rx_hz: frequency,
        tx_hz: frequency,
        note: None,
    };

    let port = dao::repeater_port::insert(c, port).await?;
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id: r.id,
            port_id: Some(port.id),
            kind: RepeaterServiceKind::Aprs,
            enabled: true,
        },
    )
    .await?;
    dao::repeater_service_aprs::insert(
        c,
        NewRepeaterServiceAprs {
            service_id: service.id,
            mode: AprsMode::Digipeater,
            path: None,
        },
    )
    .await?;

    Ok(port)
}

fn label_for_hz(tx_hz: i64) -> &'static str {
    if tx_hz < 200_000_000 {
        "VHF"
    } else if tx_hz < 1_000_000_000 {
        "UHF"
    } else {
        "SHF"
    }
}

pub async fn generate(c: &mut AsyncPgConnection) -> QueryResult<()> {
    let mut club = ham_club(c, "LA4O").await?;
    club.web_url = Some("https://la4o.no/oversikt-og-status".to_string());
    let club = dao::ham_club::update(c, club).await?;
    let club = &Some(club);

    let r = &repeater(c, &club, "LA5OR").await?;
    narrow_fm(c, r, "VHF", 145_600_000, -600_000, Some(123.0)).await?;

    let r = &repeater(c, &club, "LA7OR").await?;
    narrow_fm(c, r, "UHF", 434_775_000, -2_000_000, Some(123.0)).await?;

    let r = &repeater(c, &club, "LD1OA").await?;
    narrow_fm(c, r, "UHF", 434_887_500, -2_000_000, Some(123.0)).await?;

    let r = &repeater(c, &club, "LD1OT").await?;
    dstar(c, r, "A", 1_297_100_000, -6_000_000).await?;
    dstar(c, r, "B", 434_862_500, -2_000_000).await?;

    let r = &repeater(c, &club, "LD1OS").await?;
    igate(c, r, "A", 144_800_000).await?;

    let r = &repeater(c, &club, "LD1OE").await?;
    digipeater(c, r, "A", 144_800_000).await?;

    let mut club = ham_club(c, "LA1T").await?;
    club.web_url = Some("https://la1t.no/repeatere/".to_string());
    let club = dao::ham_club::update(c, club).await?;
    let club = &Some(club);

    {
        let mut system = repeater(c, club, "LA3XRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r#"Type: FM-crossbandlink
Kommentar: Linket til "Fylkesnett" Vestfold og Telemark"#
                .to_string(),
        );
        let site = repeater_site(c, r"Hvittingen").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_225_000),
            145_225_000,
            144_625_000 - 145_225_000,
            Some(74.4),
        )
        .await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(432_587_500),
            432_587_500,
            434_587_500 - 432_587_500,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA3SRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r#"Type: FM-crossbandlink
Kommentar: Linket til "Fylkesnett" Vestfold og Telemark"#
                .to_string(),
        );
        let site = repeater_site(c, r"Korpås (Brunlanes)").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_275_000),
            145_275_000,
            144_675_000 - 145_275_000,
            Some(74.4),
        )
        .await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(432_587_500),
            432_587_500,
            434_587_500 - 432_587_500,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA3BRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r#"Type: FM-repeater
Kommentar: Planlagt linking til "Fylkesnett" i Vestfold og Telemark"#
                .to_string(),
        );
        let site = repeater_site(c, r"Drangedal").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_562_500),
            145_562_500,
            144_962_500 - 145_562_500,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA3GRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Gaustatoppen").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_612_500),
            145_612_500,
            145_012_500 - 145_612_500,
            Some(74.4),
        )
        .await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(432_587_500),
            432_587_500,
            434_587_500 - 432_587_500,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA5HR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Horten, Skottås").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_625_000),
            145_625_000,
            145_025_000 - 145_625_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA5GR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: Linket til LA3NRR, X-bandlink i Notodden"
                .to_string(),
        );
        let site = repeater_site(c, r"Skien, Vealøs").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_650_000),
            145_650_000,
            145_050_000 - 145_650_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA5ER").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r#"Type: FM-repeater
Kommentar: Planlagt linking til "Fylkesnett" i Vestfold og Telemark"#
                .to_string(),
        );
        let site = repeater_site(c, r"Eirefjell, Tokke").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_712_500),
            145_712_500,
            145_112_500 - 145_712_500,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA5SR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Sandefjord, Mokollen").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(145_750_000),
            145_750_000,
            145_150_000 - 145_750_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA3NRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: Lokal aksess til Notodden for LA5GR"
                .to_string(),
        );
        let site = repeater_site(c, r"Notodden, Sem").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(434_825_000),
            434_825_000,
            432_825_000 - 434_825_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LD3GL").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: DMR-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Skien, Vealøs").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        let port = create_port(
            c,
            system.id,
            label_for_hz(434_512_500),
            434_512_500,
            432_512_500,
        )
        .await?;
        dmr_service_on_port(c, system.id, port.id, Some(242701)).await?;
    }

    {
        let mut system = repeater(c, club, "LD3ST").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: DMR-repeater
Kommentar: Ex. LA3KRR"
                .to_string(),
        );
        let site = repeater_site(c, r"Tønsberg, Frodeåsen").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        let port = create_port(
            c,
            system.id,
            label_for_hz(434_550_000),
            434_550_000,
            432_550_000,
        )
        .await?;
        dmr_service_on_port(c, system.id, port.id, Some(242801)).await?;
    }

    {
        let mut system = repeater(c, club, "LD3TD").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: D-Star repeater
Kommentar: Normalt linket til XRF404B"
                .to_string(),
        );
        let site = repeater_site(c, r"Tønsberg").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        let port = create_port(
            c,
            system.id,
            label_for_hz(434_562_500),
            434_562_500,
            432_562_500,
        )
        .await?;
        dstar_service_on_port(c, system.id, port.id).await?;
    }

    {
        let mut system = repeater(c, club, "LA3DRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r#"Type: FM-repeaterlink
Kommentar: Planlagt linking til "Fylkesnett" i Vestfold og Telemark"#
                .to_string(),
        );
        let site = repeater_site(c, r"Vealøs, Skien").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(434_612_500),
            434_612_500,
            432_612_500 - 434_612_500,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA3VRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM&C4FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Skien, Vealøs").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        let port = narrow_fm(
            c,
            &system,
            label_for_hz(434_587_500),
            434_587_500,
            432_587_500 - 434_587_500,
            Some(74.4),
        )
        .await?;
        c4fm_service_on_port(c, system.id, port.id).await?;
    }

    {
        let mut system = repeater(c, club, "LA6HR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM&DMR-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Holmestrand, Hvittingen").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        let port = narrow_fm(
            c,
            &system,
            label_for_hz(434_650_000),
            434_650_000,
            432_650_000 - 434_650_000,
            Some(74.4),
        )
        .await?;
        dmr_service_on_port(c, system.id, port.id, Some(242803)).await?;
    }

    {
        let mut system = repeater(c, club, "LA3JRR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Kvelde, Jordstøyp").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(434_675_000),
            434_675_000,
            432_675_000 - 434_675_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA7SR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Sandefjord, Kjerringberget").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(434_800_000),
            434_800_000,
            432_800_000 - 434_800_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA6YR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Kragerø, Storkollen").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(434_850_000),
            434_850_000,
            432_850_000 - 434_850_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA7LR").await?;
        system.status = "Midl, QRT".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Lifjell, Bø").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(434_925_000),
            434_925_000,
            432_925_000 - 434_925_000,
            Some(74.4),
        )
        .await?;
    }

    {
        let mut system = repeater(c, club, "LA9NR").await?;
        system.status = "QRV".to_string();
        system.description = Some(
            r"Type: FM-repeater
Kommentar: "
                .to_string(),
        );
        let site = repeater_site(c, r"Tønsberg, Frodeåsen").await?;
        system.site_id = Some(site.id);
        let system = dao::repeater_system::update(c, system).await?;
        narrow_fm(
            c,
            &system,
            label_for_hz(434_950_000),
            434_950_000,
            432_950_000 - 434_950_000,
            Some(74.4),
        )
        .await?;
    }

    Ok(())
}
