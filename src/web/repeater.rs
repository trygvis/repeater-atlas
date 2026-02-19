use super::AppState;
use super::auth::auth_header;
use super::map::{LinkedMapContext, MapContext, MapLink, MapRepeater, OrganizationMapContext};
use super::utils::distance_km;
use crate::service;
use crate::service::repeater_system::{Repeater, ServiceItems};
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone)]
struct LinkedRepeaterItem {
    call_sign: String,
    note: String,
}

struct NearbyRepeaterItem {
    call_sign: String,
    distance_km: f64,
    distance_label: String,
}

struct OrganizationRepeaterItem {
    call_sign: String,
    status: String,
    description: Option<String>,
    services: ServiceItems,
}

#[derive(Template)]
#[template(path = "pages/repeater.html")]
struct DetailTemplate {
    auth: super::AuthHeader,
    call_sign: String,
    repeater: Repeater,
    links: Vec<LinkedRepeaterItem>,
    nearby_repeaters: Vec<NearbyRepeaterItem>,
    status: String,
    description: Option<String>,
    maidenhead: Option<String>,
    map_context: Option<MapContext>,
    linked_map_context: Option<LinkedMapContext>,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/{call_sign}")]
pub struct CallSignDetailPath {
    pub call_sign: String,
}

pub async fn call_sign(
    CallSignDetailPath { call_sign }: CallSignDetailPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let call_sign = call_sign.to_uppercase();

    let kind = {
        let mut c = state.pool.get().await?;
        dao::call_sign::find_by_call_sign(&mut c, call_sign.clone())
            .await?
            .map(|row| row.kind)
    };

    match kind {
        Some(dao::call_sign::CallSignKind::Repeater) => {
            render_repeater_detail(call_sign, jar, state).await
        }
        Some(dao::call_sign::CallSignKind::Contact) => {
            render_contact_detail(call_sign, jar, state).await
        }
        None => Err(RepeaterAtlasError::NotFound),
    }
}

#[derive(Template)]
#[template(path = "pages/contact.html")]
struct ContactDetailTemplate {
    auth: super::AuthHeader,
    call_sign: String,
    contact: dao::contact::Contact,
    repeaters: Vec<OrganizationRepeaterItem>,
    map_context: Option<OrganizationMapContext>,
}

async fn render_contact_detail(
    call_sign: String,
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let contact = match dao::contact::find_by_call_sign(&mut c, call_sign.clone()).await? {
        Some(row) => row,
        None => return Err(RepeaterAtlasError::NotFound),
    };

    let owned_repeaters =
        dao::repeater_system::select_with_call_sign_by_owner(&mut c, contact.id).await?;
    let tech_contact_repeaters =
        dao::repeater_system::select_with_call_sign_by_tech_contact(&mut c, contact.id).await?;

    let mut repeaters_by_call_sign = BTreeMap::new();
    let mut map_repeaters_by_id: HashMap<i64, MapRepeater> =
        HashMap::with_capacity(owned_repeaters.len() + tech_contact_repeaters.len());
    let mut club_repeater_ids =
        Vec::with_capacity(owned_repeaters.len() + tech_contact_repeaters.len());
    let mut club_repeater_id_set = HashSet::new();

    for repeater in owned_repeaters
        .into_iter()
        .chain(tech_contact_repeaters.into_iter())
    {
        if !club_repeater_id_set.insert(repeater.id) {
            continue;
        }
        club_repeater_ids.push(repeater.id);

        let repeater = service::repeater_system::load(&mut c, repeater).await?;

        if let Some(point) = repeater.point {
            map_repeaters_by_id.insert(
                repeater.id,
                MapRepeater {
                    call_sign: repeater.call_sign.clone(),
                    point,
                    status: repeater.status.clone(),
                    services: Vec::new(),
                    is_external: false,
                },
            );
        }

        repeaters_by_call_sign.insert(
            repeater.call_sign.clone(),
            OrganizationRepeaterItem {
                call_sign: repeater.call_sign.clone(),
                status: repeater.status.clone(),
                description: repeater.description.clone(),
                services: repeater.services,
            },
        );
    }

    let mut map_links = Vec::new();
    if !club_repeater_ids.is_empty() {
        let links = dao::repeater_link::select_for_repeater_ids(&mut c, &club_repeater_ids).await?;
        let mut linked_ids: HashSet<i64> = HashSet::new();
        for link in &links {
            linked_ids.insert(link.repeater_a_id);
            linked_ids.insert(link.repeater_b_id);
        }

        if !linked_ids.is_empty() {
            let linked_repeaters = dao::repeater_system::select_by_ids(
                &mut c,
                &linked_ids.iter().copied().collect::<Vec<_>>(),
            )
            .await?;
            for repeater in linked_repeaters {
                if map_repeaters_by_id.contains_key(&repeater.id) {
                    continue;
                }
                let repeater_id = repeater.id;
                // External markers highlight linked repeaters outside the club set.
                // See docs/DESIGN_REPEATER.md#linking.
                let is_external = !club_repeater_id_set.contains(&repeater_id);
                if let Some(map_repeater) = map_repeater_for_display(&repeater, is_external) {
                    map_repeaters_by_id.insert(repeater_id, map_repeater);
                }
            }
        }

        map_links = build_map_links(&map_repeaters_by_id, links);
    }

    let map_repeaters = finalize_map_repeaters(map_repeaters_by_id);

    let map_context = if map_repeaters.is_empty() {
        None
    } else {
        Some(OrganizationMapContext {
            repeaters: map_repeaters,
            links: map_links,
        })
    };

    let auth = auth_header(&jar, &state);
    let template = ContactDetailTemplate {
        auth,
        call_sign,
        contact,
        repeaters: repeaters_by_call_sign.into_values().collect(),
        map_context,
    };
    let body = template.render()?;

    Ok(Html(body))
}

async fn render_repeater_detail(
    call_sign: String,
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    const NEARBY_RADIUS_METERS: f64 = 50_000.0;
    let mut c = state.pool.get().await?;

    let repeater = service::repeater_system::load_by_call_sign(&mut c, call_sign.clone()).await?;

    let links: Vec<LinkedRepeaterItem> =
        dao::repeater_link::select_with_other_call_sign(&mut c, repeater.id)
            .await?
            .into_iter()
            .map(|row| LinkedRepeaterItem {
                call_sign: row.other_call_sign,
                note: row.note,
            })
            .collect();

    // Linked network map rules: only show when more than one node exists, and
    // only link nodes with coordinates. See docs/DESIGN_REPEATER.md#linking.
    let linked_map_context = if links.is_empty() {
        None
    } else {
        let linked_paths =
            dao::repeater_link::find_linked_repeaters(&mut c, repeater.call_sign.clone()).await?;
        if linked_paths.len() <= 1 {
            None
        } else {
            let linked_call_signs: Vec<String> =
                linked_paths.into_iter().map(|row| row.call_sign).collect();
            let linked_repeaters =
                dao::repeater_system::select_by_call_signs(&mut c, &linked_call_signs).await?;
            let linked_ids: Vec<i64> = linked_repeaters
                .iter()
                .map(|repeater| repeater.id)
                .collect();
            let mut map_repeaters_by_id: HashMap<i64, MapRepeater> = HashMap::new();

            for repeater in linked_repeaters {
                if let Some(map_repeater) = map_repeater_for_display(&repeater, false) {
                    map_repeaters_by_id.insert(repeater.id, map_repeater);
                }
            }

            if map_repeaters_by_id.is_empty() {
                None
            } else {
                let link_rows =
                    dao::repeater_link::select_for_repeater_ids(&mut c, &linked_ids).await?;
                let map_links = build_map_links(&map_repeaters_by_id, link_rows);

                let map_repeaters = finalize_map_repeaters(map_repeaters_by_id);

                Some(LinkedMapContext {
                    repeaters: map_repeaters,
                    links: map_links,
                })
            }
        }
    };

    let status = repeater.status.clone();
    let description = repeater.description.clone();
    let maidenhead = repeater.maidenhead.as_ref().map(|value| value.to_string());
    let point = repeater.point;
    let (map_context, nearby_repeaters) = if let Some(center) = point {
        let all_repeaters =
            dao::repeater_system::select_within_radius(&mut c, center, NEARBY_RADIUS_METERS)
                .await?;
        let mut nearby_repeaters = Vec::new();
        let mut nearby_list = Vec::new();

        for candidate in all_repeaters {
            if let Some(candidate_point) = candidate.location() {
                let distance = distance_km(center, candidate_point);
                nearby_list.push(NearbyRepeaterItem {
                    call_sign: candidate.call_sign.clone(),
                    distance_km: distance,
                    distance_label: format!("{distance:.1} km"),
                });

                nearby_repeaters.push(MapRepeater {
                    call_sign: candidate.call_sign,
                    point: candidate_point,
                    status: candidate.status,
                    services: Vec::new(),
                    is_external: false,
                });
            }
        }

        nearby_list.sort_by(|a, b| {
            a.distance_km
                .partial_cmp(&b.distance_km)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.call_sign.cmp(&b.call_sign))
        });

        (
            Some(MapContext {
                center,
                radius_meters: NEARBY_RADIUS_METERS as u32,
                repeaters: nearby_repeaters,
            }),
            nearby_list,
        )
    } else {
        (None, Vec::new())
    };

    let auth = auth_header(&jar, &state);
    let template = DetailTemplate {
        auth,
        call_sign,
        repeater,
        links,
        nearby_repeaters,
        status,
        description,
        maidenhead,
        map_context,
        linked_map_context,
    };
    let body = template.render()?;

    Ok(Html(body))
}

fn map_repeater_for_display(
    repeater: &dao::repeater_system::RepeaterSystemDao,
    is_external: bool,
) -> Option<MapRepeater> {
    let Some(point) = repeater.location() else {
        return None;
    };

    Some(MapRepeater {
        call_sign: repeater.call_sign.clone(),
        point,
        status: repeater.status.clone(),
        services: Vec::new(),
        is_external,
    })
}

fn build_map_links(
    map_repeaters_by_id: &HashMap<i64, MapRepeater>,
    links: Vec<dao::repeater_link::RepeaterLink>,
) -> Vec<MapLink> {
    let mut map_links = Vec::new();

    for link in links {
        if let (Some(a), Some(b)) = (
            map_repeaters_by_id.get(&link.repeater_a_id),
            map_repeaters_by_id.get(&link.repeater_b_id),
        ) {
            map_links.push(MapLink {
                from: a.point.clone(),
                to: b.point.clone(),
                from_call_sign: a.call_sign.clone(),
                to_call_sign: b.call_sign.clone(),
            });
        }
    }

    map_links
}

fn finalize_map_repeaters(map_repeaters_by_id: HashMap<i64, MapRepeater>) -> Vec<MapRepeater> {
    let mut map_repeaters: Vec<MapRepeater> = map_repeaters_by_id.into_values().collect();
    map_repeaters.sort_by(|a, b| a.call_sign.cmp(&b.call_sign));
    map_repeaters
}
