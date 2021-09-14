macro_rules! return_inner {
    ($s:ident => use $u:ident,lock $l:ident,ret $r:ident,data $d:ident) => {{
        let guard = $s.$l.lock().await;

        $s.$u($d).await?;
        let ret = $s.$r.lock().await.recv().await.unwrap();

        drop(guard);

        Ok(ret)
    }};
}

pub mod content;
pub mod user;
