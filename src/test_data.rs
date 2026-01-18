use crate::dao;
use crate::dao::ham_club::NewHamClub;
use crate::dao::repeater::NewRepeater;
use crate::dao::repeater_port::NewRepeaterPort;
use diesel::QueryResult;
use diesel_async::AsyncPgConnection;

struct PortSpec {
    label: &'static str,
    tx_hz: i64,
    rx_offset_hz: i64,
}

impl PortSpec {
    fn new(label: &'static str, tx_hz: i64, rx_offset_hz: i64) -> Self {
        Self {
            label,
            tx_hz,
            rx_offset_hz,
        }
    }

    fn rx_hz(&self) -> i64 {
        self.tx_hz + self.rx_offset_hz
    }
}

pub async fn generate(c: &mut AsyncPgConnection) -> QueryResult<()> {
    let la4o = dao::ham_club::insert(c, NewHamClub::new("LA4O")).await?;

    let repeaters = [
        (
            "LA5OR",
            vec![PortSpec::new("VHF", 145_600_000, -600_000)],
        ),
        (
            "LA7OR",
            vec![PortSpec::new("UHF", 434_775_000, -2_000_000)],
        ),
        (
            "LD1OA",
            vec![PortSpec::new("UHF", 434_887_500, -2_000_000)],
        ),
        (
            // D-STAR system with multiple RF modules/ports.
            "LD1OT",
            vec![
                PortSpec::new("A", 1_297_100_000, -6_000_000),
                PortSpec::new("B", 434_862_500, -2_000_000),
            ],
        ),
        ("LD1OS", vec![PortSpec::new("APRS", 144_800_000, 0)]),
        ("LD1OE", vec![PortSpec::new("APRS", 144_800_000, 0)]),
    ];

    for (call_sign, ports) in repeaters {
        let repeater = dao::repeater::insert(c, NewRepeater::new(call_sign).ham_club_id(la4o.id))
            .await?;

        for port in ports {
            dao::repeater_port::insert(
                c,
                NewRepeaterPort::new(repeater.id, port.label, port.rx_hz(), port.tx_hz),
            )
            .await?;
        }
    }

    Ok(())
}

