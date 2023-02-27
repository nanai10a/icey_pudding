use mongodb::bson::doc;
use mongodb::error::Result as MongoResult;
use mongodb::options::{Acknowledgment, ReadConcern, TransactionOptions, WriteConcern};
use mongodb::{Client, ClientSession, Collection, Database};
use tracing::Instrument;

use super::converters::{convert_404_or, convert_repo_err, to_bool};
use super::Result as RepoResult;
use crate::utils::LetChain;

pub async fn initialize_coll(
    coll_name: impl Into<::mongodb::bson::Bson>,
    db: &Database,
) -> MongoResult<()> {
    db.run_command(
        doc! {
            "createIndexes": coll_name.into(),
            "indexes": [{
                "name": "unique_id",
                "key": {
                    "id": 1
                },
                "unique": true
            }],
        },
        None,
    )
    .instrument(tracing::trace_span!("run_command"))
    .await?;

    Ok(())
}

pub async fn make_session(c: &Client) -> MongoResult<ClientSession> {
    let mut s = c
        .start_session(None)
        .instrument(tracing::trace_span!("start_session"))
        .await?;

    let ta_opt = TransactionOptions::builder()
        .read_concern(ReadConcern::snapshot())
        .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
        .build();
    s.start_transaction(ta_opt)
        .instrument(tracing::trace_span!("start_transaction"))
        .await?;

    Ok(s)
}

pub async fn process_transaction(s: &mut ClientSession) -> MongoResult<()> {
    loop {
        let r = s
            .commit_transaction()
            .instrument(tracing::trace_span!("commit_transaction"))
            .await;
        if let Err(ref e) = r {
            if e.contains_label(::mongodb::error::UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                continue;
            }
        }

        break r;
    }
}

pub async fn exec_transaction<F, I, FO, RO>(f: F, arg: I) -> MongoResult<RO>
where
    F: Fn<I, Output = FO>,
    I: Clone + ::core::marker::Tuple,
    FO: ::core::future::Future<Output = MongoResult<RO>>,
{
    loop {
        let r = f.call(arg.clone()).await;
        if let Err(ref e) = r {
            if e.contains_label(::mongodb::error::TRANSIENT_TRANSACTION_ERROR) {
                continue;
            }

            break r;
        }
    }
}

pub async fn get_set<T>(
    coll: &Collection<T>,
    id: impl Into<::mongodb::bson::Bson>,
) -> RepoResult<T>
where
    T: Sync + Send + Unpin + ::serde::de::DeserializeOwned,
{
    let res = coll
        .find_one(doc! { "id": id.into() }, None)
        .instrument(tracing::trace_span!("find_one"))
        .await
        .let_(convert_repo_err)?
        .let_(convert_404_or)?;

    Ok(res)
}

pub async fn is_contains<T>(
    name: impl AsRef<str>,
    coll: &Collection<T>,
    id: impl Into<::mongodb::bson::Bson>,
    target: impl Into<::mongodb::bson::Bson>,
) -> RepoResult<bool> {
    let res = coll
        .count_documents(
            doc! {
                "id": id.into(),
                name.as_ref(): { "$in": [target.into()] }
            },
            None,
        )
        .instrument(tracing::trace_span!("count_documents"))
        .await
        .let_(convert_repo_err)?
        .let_(to_bool);

    Ok(res)
}

#[derive(Clone, Copy)]
pub enum ModifyOpTy {
    Push,
    Pull,
}

pub async fn modify_set<T>(
    name: impl AsRef<str>,
    coll: &Collection<T>,
    client: &Client,
    id: impl Into<::mongodb::bson::Bson>,
    target: impl Into<::mongodb::bson::Bson>,
    ty: ModifyOpTy,
) -> RepoResult<bool> {
    async fn transaction<T>(
        name: &str,
        coll: &Collection<T>,
        client: &Client,
        id: &::mongodb::bson::Bson,
        target: &::mongodb::bson::Bson,
        ty: ModifyOpTy,
    ) -> MongoResult<Option<bool>> {
        let mut session = make_session(client).await?;

        let operation = match ty {
            ModifyOpTy::Push => "$addToSet",
            ModifyOpTy::Pull => "$pull",
        };
        let res = coll
            .update_one_with_session(
                doc! { "id": id },
                doc! { operation: { name: target } },
                None,
                &mut session,
            )
            .instrument(tracing::trace_span!("update_one_with_session"))
            .await?;

        if !res.matched_count.let_(to_bool) {
            return Ok(None);
        };
        if !res.modified_count.let_(to_bool) {
            return Ok(Some(false));
        }

        let inc_name = &format!("{}_size", name);
        let inc_value = match ty {
            ModifyOpTy::Push => 1,
            ModifyOpTy::Pull => -1,
        };
        let res = coll
            .update_one_with_session(
                doc! { "id": id },
                doc! { "$inc": { inc_name: inc_value } },
                None,
                &mut session,
            )
            .instrument(tracing::trace_span!("update_one_with_session"))
            .await?;

        if !res.matched_count.let_(to_bool) {
            unreachable!("not found value");
        }
        if !res.modified_count.let_(to_bool) {
            let op = match ty {
                ModifyOpTy::Push => "inc",
                ModifyOpTy::Pull => "dec",
            };
            unreachable!("cannot {} {} field", op, inc_name);
        }

        process_transaction(&mut session).await.map(|_| Some(true))
    }

    let id_bson = id.into();
    let target_bson = target.into();

    let res = exec_transaction(
        transaction,
        (name.as_ref(), coll, client, &id_bson, &target_bson, ty),
    )
    .await;
    res.let_(convert_repo_err)?.let_(convert_404_or)
}
