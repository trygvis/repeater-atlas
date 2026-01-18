use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ham_club)]
pub struct NewHamClub {
    pub name: String,
    pub description: Option<String>,
    pub web_url: Option<String>,
    pub email: Option<String>,
}

impl NewHamClub {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            web_url: None,
            email: None,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::ham_club)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HamClub {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub web_url: Option<String>,
    pub email: Option<String>,
}

pub async fn insert(c: &mut AsyncPgConnection, new_club: NewHamClub) -> QueryResult<HamClub> {
    use crate::schema::ham_club::dsl as h;

    diesel::insert_into(h::ham_club)
        .values(&new_club)
        .returning(HamClub::as_returning())
        .get_result(c)
        .await
}

pub async fn select(c: &mut AsyncPgConnection) -> QueryResult<Vec<HamClub>> {
    use crate::schema::ham_club::dsl as h;

    h::ham_club
        .select(HamClub::as_select())
        .get_results(c)
        .await
}

pub async fn get(c: &mut AsyncPgConnection, club_id: i64) -> QueryResult<HamClub> {
    use crate::schema::ham_club::dsl as h;

    h::ham_club
        .filter(h::id.eq(club_id))
        .select(HamClub::as_select())
        .first(c)
        .await
}

pub async fn find_by_name(
    c: &mut AsyncPgConnection,
    club_name: String,
) -> QueryResult<Option<HamClub>> {
    use crate::schema::ham_club::dsl as h;

    h::ham_club
        .filter(h::name.eq(club_name))
        .select(HamClub::as_select())
        .first(c)
        .await
        .optional()
}
