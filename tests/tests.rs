use tsukuyomi_router::Router;

#[test]
fn simple() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/domain/mime", "mime")?;
    router.add_route("/domain/yours", "yours")?;

    assert_eq!(
        router
            .recognize("/domain/mime")
            .route()
            .map(|(r, _)| r.data()),
        Some(&"mime")
    );
    assert_eq!(
        router
            .recognize("/domain/yours")
            .route()
            .map(|(r, _)| r.data()),
        Some(&"yours")
    );

    assert!(router.recognize("/domain/me").route().is_none());

    Ok(())
}

#[test]
fn param() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/posts/:post", "the_post")?;
    router.add_route("/posts/create", "create_post")?;

    assert_eq!(
        router
            .recognize("/posts/create")
            .route()
            .map(|(r, _)| r.data()),
        Some(&"create_post")
    );

    let res = router.recognize("/posts/12");
    if let Some((route, Some(params))) = res.route() {
        assert_eq!(*route.data(), "the_post");
        assert_eq!(params.get(0), Some("12"));
        assert_eq!(params.name("post"), Some("12"));
    } else {
        panic!("unexpected condition");
    }

    Ok(())
}

#[test]
fn wildcard() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/public/*/index.html", "public_index")?;
    router.add_route("/*", "catch_all")?;

    let res = router.recognize("/public/path/to/index.html");
    if let Some((route, Some(params))) = res.route() {
        assert_eq!(*route.data(), "public_index");
        assert_eq!(params.get_wildcard(), Some("path/to"));
        assert_eq!(params.name("*"), Some("path/to"));
    } else {
        panic!("unexpected condition");
    }

    let res = router.recognize("/path/to/index.html");
    if let Some((route, Some(params))) = res.route() {
        assert_eq!(*route.data(), "catch_all");
        assert_eq!(params.get_wildcard(), Some("path/to/index.html"));
        assert_eq!(params.name("*"), Some("path/to/index.html"));
    } else {
        panic!("unexpected condition");
    }

    Ok(())
}

#[test]
fn scope() -> tsukuyomi_router::Result<()> {
    let mut router = Router::new();
    router.add_route("/api/v1/posts/new", "new_post")?;
    router.add_scope("/api/v1/posts/", "posts")?;
    router.add_scope("/api/v1/", "api_v1")?;

    {
        let res = router.recognize("/api/v1/posts/new");
        assert_eq!(res.route().map(|(r, _)| r.data()), Some(&"new_post"));
        assert_eq!(res.scope().map(|(s, _)| s.data()), Some(&"posts"));
    }

    {
        let res = router.recognize("/api/v1/posts/");
        assert!(res.route().is_none());
        assert_eq!(res.scope().map(|(s, _)| s.data()), Some(&"posts"));
    }

    {
        let res = router.recognize("/api/v1/users");
        assert!(res.route().is_none());
        assert_eq!(res.scope().map(|(s, _)| s.data()), Some(&"api_v1"));
    }

    assert!(router.recognize("/favicon.ico").scope().is_none());

    Ok(())
}
