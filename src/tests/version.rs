extern crate diesel;

use rustc_serialize::json::Json;

use conduit::{Handler, Method};
use self::diesel::prelude::*;

use cargo_registry::version::EncodableVersion;
use cargo_registry::schema::versions;

#[derive(RustcDecodable)]
struct VersionList { versions: Vec<EncodableVersion> }
#[derive(RustcDecodable)]
struct VersionResponse { version: EncodableVersion }

#[test]
fn index() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app.clone(), Method::Get, "/api/v1/versions");
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionList = ::json(&mut response);
    assert_eq!(json.versions.len(), 0);

    let (v1, v2) = {
        let conn = app.diesel_database.get().unwrap();
        let u = ::new_user("foo").create_or_update(&conn).unwrap();
        ::CrateBuilder::new("foo_vers_index", u.id)
            .version("2.0.0")
            .version("2.0.1")
            .expect_build(&conn);
        let ids = versions::table.select(versions::id).load::<i32>(&*conn).unwrap();
        (ids[0], ids[1])
    };
    req.with_query(&format!("ids[]={}&ids[]={}", v1, v2));
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionList = ::json(&mut response);
    assert_eq!(json.versions.len(), 2);
}

#[test]
fn show() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app.clone(), Method::Get, "/api/v1/versions");
    let v = {
        let conn = app.diesel_database.get().unwrap();
        let user = ::new_user("foo").create_or_update(&conn).unwrap();
        let krate = ::CrateBuilder::new("foo_vers_show", user.id)
            .expect_build(&conn);
        ::new_version(krate.id, "2.0.0").save(&conn, &[]).unwrap()
    };
    req.with_path(&format!("/api/v1/versions/{}", v.id));
    let mut response = ok_resp!(middle.call(&mut req));
    let json: VersionResponse = ::json(&mut response);
    assert_eq!(json.version.id, v.id);
}

#[test]
fn authors() {
    let (_b, app, middle) = ::app();
    let mut req = ::req(app, Method::Get, "/api/v1/crates/foo_authors/1.0.0/authors");
    ::mock_user(&mut req, ::user("foo"));
    ::mock_crate(&mut req, ::krate("foo_authors"));
    let mut response = ok_resp!(middle.call(&mut req));
    let mut data = Vec::new();
    response.body.write_body(&mut data).unwrap();
    let s = ::std::str::from_utf8(&data).unwrap();
    let json = Json::from_str(&s).unwrap();
    let json = json.as_object().unwrap();
    assert!(json.contains_key(&"users".to_string()));
}
