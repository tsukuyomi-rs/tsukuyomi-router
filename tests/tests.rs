use tsukuyomi_router::Router;

#[test]
fn simple() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/domain/mime", "mime")?;
    router.add_route("/domain/yours", "yours")?;

    assert_eq!(router.find_route("/domain/mime").data, Some(&"mime"));
    assert_eq!(router.find_route("/domain/yours").data, Some(&"yours"));

    assert!(router.find_route("/domain/me").data.is_none());

    Ok(())
}

#[test]
fn param() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/posts/:post", "the_post")?;
    router.add_route("/posts/create", "create_post")?;

    assert_eq!(
        router.find_route("/posts/create").data,
        Some(&"create_post")
    );

    let res = router.find_route("/posts/12");
    assert_eq!(res.data, Some(&"the_post"));
    assert_eq!(&res.params[0], "12");

    Ok(())
}

#[test]
fn wildcard() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/public/*/index.html", "public_index")?;
    router.add_route("/*", "catch_all")?;

    {
        let res = router.find_route("/public/path/to/index.html");
        assert_eq!(res.data, Some(&"public_index"));
        assert_eq!(res.wildcard, Some("path/to".into()));
    }

    {
        let res = router.find_route("/path/to/index.html");
        assert_eq!(res.data, Some(&"catch_all"));
        assert_eq!(res.wildcard, Some("path/to/index.html".into()));
    }

    Ok(())
}
