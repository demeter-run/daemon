use anyhow::Result;
use std::path::Path;

pub struct FabricState {
    db: sqlx::sqlite::SqlitePool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ApiKey {
    pub digest: Vec<u8>,
    pub salt: Vec<u8>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ListResourceProj {
    pub name: String,
    pub uuid: Vec<u8>,
    pub kind: String,
}

pub struct AccountDelta {
    pub account: i64,
    pub debit: Option<i64>,
    pub credit: Option<i64>,
}

impl FabricState {
    pub async fn open(path: &Path) -> Result<Self> {
        let url = format!("sqlite:{}?mode=rwc", path.display());
        let db = sqlx::sqlite::SqlitePoolOptions::new().connect(&url).await?;

        Ok(Self { db })
    }

    pub async fn ephemeral() -> Result<Self> {
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;

        let out = Self { db };
        out.migrate().await?;

        Ok(out)
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("src/driven/fabric_state/migrations")
            .run(&self.db)
            .await?;

        Ok(())
    }

    pub async fn insert_namespace(&self, name: &str) -> Result<()> {
        sqlx::query!(
            r#"
INSERT INTO namespaces (name) 
VALUES ($1)
"#,
            name,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn namespace_exists(&self, name: &str) -> Result<bool> {
        let record = sqlx::query!(
            r#"
SELECT *
FROM namespaces
WHERE name = $1
"#,
            name,
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(record.is_some())
    }

    pub async fn insert_api_key(&self, ns: &str, digest: &[u8], salt: &[u8]) -> Result<()> {
        sqlx::query!(
            r#"
INSERT INTO apikeys (namespace, digest, salt) 
VALUES ($1, $2, $3)
"#,
            ns,
            digest,
            salt
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn get_all_api_keys_for_namespace(&self, ns: &str) -> Result<Vec<ApiKey>> {
        let rows = sqlx::query_as::<_, ApiKey>(
            r#"
SELECT digest, salt
FROM apikeys
WHERE namespace = $1
"#,
        )
        .bind(ns)
        .fetch_all(&self.db)
        .await?;

        Ok(rows)
    }

    pub async fn insert_resource(
        &self,
        ns: &str,
        kind: &str,
        uuid: &[u8],
        name: &str,
        manifest: &[u8],
    ) -> Result<()> {
        sqlx::query!(
            r#"
INSERT INTO resources (namespace, kind, uuid, name, manifest) 
VALUES ($1, $2, $3, $4, $5)
"#,
            ns,
            kind,
            uuid,
            name,
            manifest
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn insert_accounting(
        &self,
        epoch: i64,
        entry: &[u8],
        cluster: &[u8],
        namespace: &str,
        resource: Option<&[u8]>,
        deltas: Vec<AccountDelta>,
    ) -> Result<()> {
        let mut tx = self.db.begin().await?;

        for AccountDelta {
            account,
            debit,
            credit,
        } in deltas
        {
            sqlx::query!(
                r#"
INSERT INTO accounting (epoch, entry, cluster, namespace, resource, account, debit, credit) 
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
"#,
                epoch,
                entry,
                cluster,
                namespace,
                resource,
                account,
                debit,
                credit,
            )
            .execute(&mut *tx)
            .await?;
        }

        Ok(())
    }

    pub async fn list_resources(&self, ns: &str) -> Result<Vec<ListResourceProj>> {
        let rows = sqlx::query_as::<_, ListResourceProj>(
            r#"
SELECT name, uuid, kind FROM resources
WHERE namespace = $1
"#,
        )
        .bind(ns)
        .fetch_all(&self.db)
        .await?;

        Ok(rows)
    }

    pub async fn read_balance(&self, ns: &str) -> Result<Vec<(i64, i64, i64)>> {
        let rows = sqlx::query_as::<_, (i64, i64, i64)>(
            r#"
SELECT account, sum(debit), sum(credit) FROM accounting
WHERE namespace = $1
GROUP BY account
"#,
        )
        .bind(ns)
        .fetch_all(&self.db)
        .await?;

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_namespace_persistence() {
        let db = FabricState::ephemeral().await.unwrap();

        db.migrate().await.unwrap();

        assert_eq!(db.namespace_exists("ns1").await.unwrap(), false);

        db.insert_namespace("ns1").await.unwrap();

        assert_eq!(db.namespace_exists("ns1").await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_apikeys_persistence() {
        let db = FabricState::ephemeral().await.unwrap();

        db.migrate().await.unwrap();

        db.insert_namespace("ns1").await.unwrap();
        db.insert_api_key("ns1", b"0123", b"9876").await.unwrap();
        db.insert_api_key("ns1", b"4567", b"5432").await.unwrap();

        db.insert_namespace("ns2").await.unwrap();
        db.insert_api_key("ns2", b"abcd", b"zyxw").await.unwrap();

        // TODO: don't fail if results are return in different order
        let mut keys = db.get_all_api_keys_for_namespace("ns1").await.unwrap();
        assert_eq!(keys.len(), 2);
        let item = keys.remove(0);
        assert_eq!(item.digest, b"0123");
        assert_eq!(item.salt, b"9876");
        let item = keys.remove(0);
        assert_eq!(item.digest, b"4567");
        assert_eq!(item.salt, b"5432");

        let mut keys = db.get_all_api_keys_for_namespace("ns2").await.unwrap();
        assert_eq!(keys.len(), 1);
        let item = keys.remove(0);
        assert_eq!(item.digest, b"abcd");
        assert_eq!(item.salt, b"zyxw");
    }

    #[tokio::test]
    async fn test_accounting_persistence() {
        let db = FabricState::ephemeral().await.unwrap();

        db.migrate().await.unwrap();

        db.insert_namespace("ns1").await.unwrap();

        db.insert_resource("ns1", "pod", b"resource1", "mypod", b"")
            .await
            .unwrap();

        db.insert_accounting(
            1,
            b"entry1",
            b"cluster1",
            "ns1",
            Some(b"resource1"),
            vec![
                AccountDelta {
                    account: 1,
                    debit: Some(400),
                    credit: None,
                },
                AccountDelta {
                    account: 2,
                    debit: None,
                    credit: Some(400),
                },
            ],
        )
        .await
        .unwrap();

        db.insert_accounting(
            1,
            b"entry1",
            b"cluster1",
            "ns1",
            None,
            vec![
                AccountDelta {
                    account: 1,
                    debit: Some(400),
                    credit: None,
                },
                AccountDelta {
                    account: 2,
                    debit: None,
                    credit: Some(400),
                },
            ],
        )
        .await
        .unwrap();

        let mut balance = db.read_balance("ns1").await.unwrap();

        print!("{:?}", balance);

        let entry1 = balance.remove(0);
        assert_eq!(entry1, (1, 0, 0));
    }
}
