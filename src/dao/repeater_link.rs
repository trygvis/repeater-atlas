use crate::RepeaterAtlasError;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Array, BigInt, Int4, Text};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_link)]
pub struct NewRepeaterLink {
    pub repeater_a_id: i64,
    pub repeater_b_id: i64,
    pub note: String,
}

impl NewRepeaterLink {
    pub fn new(repeater_a_id: i64, repeater_b_id: i64) -> Self {
        Self {
            repeater_a_id,
            repeater_b_id,
            note: String::new(),
        }
    }
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::repeater_link)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterLink {
    pub id: i64,
    pub repeater_a_id: i64,
    pub repeater_b_id: i64,
    pub note: String,
}

pub async fn insert(c: &mut AsyncPgConnection, link: NewRepeaterLink) -> QueryResult<usize> {
    use crate::schema::repeater_link::dsl as l;

    let mut link = link;
    // Normalize order to satisfy (a<b) constraint for undirected links.
    if link.repeater_a_id > link.repeater_b_id {
        std::mem::swap(&mut link.repeater_a_id, &mut link.repeater_b_id);
    }

    diesel::insert_into(l::repeater_link)
        .values(&link)
        .on_conflict((l::repeater_a_id, l::repeater_b_id))
        .do_nothing()
        .execute(c)
        .await
}

pub async fn select_by_repeater_id(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
) -> QueryResult<Vec<RepeaterLink>> {
    use crate::schema::repeater_link::dsl as l;

    l::repeater_link
        .filter(
            l::repeater_a_id
                .eq(repeater_id)
                .or(l::repeater_b_id.eq(repeater_id)),
        )
        .select(RepeaterLink::as_select())
        .order_by((l::repeater_a_id.asc(), l::repeater_b_id.asc()))
        .get_results(c)
        .await
}

#[derive(QueryableByName)]
pub struct RepeaterLinkWithOtherCallSign {
    #[diesel(sql_type = BigInt)]
    pub other_repeater_id: i64,
    #[diesel(sql_type = Text)]
    pub other_call_sign: String,
    #[diesel(sql_type = Text)]
    pub note: String,
}

pub async fn select_with_other_call_sign(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
) -> QueryResult<Vec<RepeaterLinkWithOtherCallSign>> {
    // Diesel DSL cannot express the conditional "other" side selection + join cleanly.
    let rows = sql_query(
        r#"
        SELECT
            CASE
                WHEN rl.repeater_a_id = $1 THEN rl.repeater_b_id
                ELSE rl.repeater_a_id
            END AS other_repeater_id,
            rs.call_sign AS other_call_sign,
            rl.note
        FROM repeater_link rl
        JOIN repeater_system rs
          ON rs.id = CASE
                         WHEN rl.repeater_a_id = $1 THEN rl.repeater_b_id
                         ELSE rl.repeater_a_id
                     END
        WHERE rl.repeater_a_id = $1 OR rl.repeater_b_id = $1
        ORDER BY rs.call_sign ASC
        "#,
    )
    .bind::<BigInt, _>(repeater_id)
    .get_results::<RepeaterLinkWithOtherCallSign>(c)
    .await?;

    Ok(rows)
}

#[derive(Debug, QueryableByName)]
pub struct LinkedRepeaterPath {
    #[diesel(sql_type = Text)]
    pub call_sign: String,
    #[diesel(sql_type = Int4)]
    pub depth: i32,
    #[diesel(sql_type = Array<Text>)]
    pub path: Vec<String>,
}

pub async fn find_linked_repeaters(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> Result<Vec<LinkedRepeaterPath>, RepeaterAtlasError> {
    let rows = sql_query(
        r#"WITH RECURSIVE
    initial AS (SELECT id, call_sign
                FROM repeater_system
                WHERE call_sign = $1),
    linked(repeater_id, call_sign, depth, path) AS ( --
        SELECT id, call_sign, 0, ARRAY [call_sign]::TEXT[]
        FROM initial
        UNION ALL
        SELECT CASE
                   WHEN rl.repeater_a_id = linked.repeater_id
                       THEN rl.repeater_b_id
                   ELSE rl.repeater_a_id
                   END AS repeater_id,
               rs.call_sign,
               linked.depth + 1,
               linked.path || rs.call_sign
        FROM linked
                 JOIN repeater_link rl
                      ON rl.repeater_a_id = linked.repeater_id
                          OR rl.repeater_b_id = linked.repeater_id
                 JOIN repeater_system rs
                      ON rs.id = CASE
                                     WHEN rl.repeater_a_id = linked.repeater_id
                                         THEN rl.repeater_b_id
                                     ELSE rl.repeater_a_id
                          END
        WHERE NOT (rs.call_sign = ANY (linked.path)))
SELECT call_sign, depth, path
FROM linked
ORDER BY depth, call_sign
"#,
    )
    .bind::<Text, _>(call_sign)
    .get_results::<LinkedRepeaterPath>(c)
    .await
    .map_err(|e| RepeaterAtlasError::DatabaseOther(e, "find_linked_repeaters".to_string()))?;

    Ok(rows)
}
