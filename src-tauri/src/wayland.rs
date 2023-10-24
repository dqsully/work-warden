use wayland_client::{
    protocol::{wl_registry, wl_seat},
    Connection, Dispatch, QueueHandle,
};

use self::idle::{org_kde_kwin_idle, org_kde_kwin_idle_timeout};

pub mod idle {
    #![allow(non_upper_case_globals, non_camel_case_types)]
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("./wayland-protocols/idle.xml");
    }
    use self::__interfaces::*;

    wayland_scanner::generate_client_code!("./wayland-protocols/idle.xml");
}

struct IdleTracker<F>
where
    F: Fn(bool),
{
    callback: F,

    wl_seat: Option<wl_seat::WlSeat>,
    kwin_idle: Option<org_kde_kwin_idle::OrgKdeKwinIdle>,
    kwin_idle_timeout: Option<org_kde_kwin_idle_timeout::OrgKdeKwinIdleTimeout>,
}

impl<F> Dispatch<wl_registry::WlRegistry, ()> for IdleTracker<F>
where
    F: Fn(bool) + 'static,
{
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            let mut idle_registration = false;

            match &interface[..] {
                "wl_seat" => {
                    state.wl_seat =
                        Some(registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ()));
                    idle_registration = true;
                }
                "org_kde_kwin_idle" => {
                    state.kwin_idle =
                        Some(registry.bind::<org_kde_kwin_idle::OrgKdeKwinIdle, _, _>(
                            name,
                            version,
                            qh,
                            (),
                        ));
                    idle_registration = true;
                }
                _ => {}
            }

            if idle_registration {
                if let (Some(wl_seat), Some(kwin_idle)) = (&state.wl_seat, &state.kwin_idle) {
                    state.kwin_idle_timeout =
                        Some(kwin_idle.get_idle_timeout(wl_seat, 300_000, qh, ()));
                }
            }
        }
    }
}

impl<F> Dispatch<wl_seat::WlSeat, ()> for IdleTracker<F>
where
    F: Fn(bool),
{
    fn event(
        _state: &mut Self,
        _seat: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl<F> Dispatch<org_kde_kwin_idle::OrgKdeKwinIdle, ()> for IdleTracker<F>
where
    F: Fn(bool),
{
    fn event(
        _state: &mut Self,
        _seat: &org_kde_kwin_idle::OrgKdeKwinIdle,
        _event: org_kde_kwin_idle::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl<F> Dispatch<org_kde_kwin_idle_timeout::OrgKdeKwinIdleTimeout, ()> for IdleTracker<F>
where
    F: Fn(bool),
{
    fn event(
        state: &mut Self,
        _timeout: &org_kde_kwin_idle_timeout::OrgKdeKwinIdleTimeout,
        event: <org_kde_kwin_idle_timeout::OrgKdeKwinIdleTimeout as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            org_kde_kwin_idle_timeout::Event::Idle => (state.callback)(true),
            org_kde_kwin_idle_timeout::Event::Resumed => (state.callback)(false),
        }
    }
}

pub fn listen_idle<F>(callback: F)
where
    F: Fn(bool) + Send + 'static,
{
    std::thread::spawn(move || {
        let conn = Connection::connect_to_env().unwrap();

        let display = conn.display();

        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());

        let mut tracker = IdleTracker {
            callback,
            wl_seat: None,
            kwin_idle: None,
            kwin_idle_timeout: None,
        };

        loop {
            event_queue.blocking_dispatch(&mut tracker).unwrap();
        }
    });
}
