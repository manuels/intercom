extern crate gcc;
extern crate pkg_config;

fn main() {
    let libgio = pkg_config::find_library("gio-2.0").unwrap();
    let libglib = pkg_config::find_library("glib-2.0").unwrap();
    let libgiounix = pkg_config::find_library("gio-unix-2.0").unwrap();

    gcc::Config::new()
            .file("src/network-bindings.c")
            .include(&libgio.include_paths[0])
            .include(&libgio.include_paths[1])
            .include(&libgiounix.include_paths[0])
            .include(&libgiounix.include_paths[1])
            .include(&libglib.include_paths[0])
            .include(&libglib.include_paths[1])
            .compile("libnetwork.a");

    gcc::Config::new()
            .file("src/intercom-dbus-bindings.c")
            .include(&libgio.include_paths[0])
            .include(&libgio.include_paths[1])
            .include(&libgiounix.include_paths[0])
            .include(&libgiounix.include_paths[1])
            .include(&libglib.include_paths[0])
            .include(&libglib.include_paths[1])
            .compile("libintercom-dbus.a");
}
