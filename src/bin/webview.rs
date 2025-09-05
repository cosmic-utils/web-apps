use clap::Parser;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::unix::EventLoopBuilderExtUnix,
    window::{WindowAttributes, WindowBuilder},
};
use wry::{
    dpi::{LogicalSize, Size},
    WebContext, WebViewBuilder,
};

fn main() -> wry::Result<()> {
    let args = webapps::WebviewArgs::parse();

    gtk::init().unwrap();

    gtk::glib::set_program_name(args.id.clone().into());
    gtk::glib::set_application_name(&args.id);

    let browser = webapps::browser::Browser::from_appid(&args.id).unwrap();

    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

    let mut attrs = WindowAttributes::default();
    if let Some(size) = browser.window_size {
        attrs.inner_size = Some(Size::new(LogicalSize::new(size.0, size.1)));
    }

    let mut window_builder = WindowBuilder::new();
    window_builder.window = attrs;

    let window = window_builder
        .with_title(browser.window_title.unwrap_or(webapps::fl!("app")))
        .with_decorations(browser.window_decorations.unwrap_or(true))
        .build(&event_loop)
        .unwrap();

    let mut context = WebContext::new(browser.profile);

    let mut builder = WebViewBuilder::new_with_web_context(&mut context)
        .with_url(browser.url.unwrap_or_default())
        .with_incognito(browser.private_mode.unwrap_or(false))
        .with_new_window_req_handler(|url, features| {
            println!("new window req: {url} {features:?}");
            wry::NewWindowResponse::Allow
        });

    if let Some(simulate) = browser.try_simulate_mobile {
        if simulate {
            builder = builder.with_user_agent(webapps::MOBILE_UA);
        }
    };

    let _webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;
        let vbox = window.default_vbox().unwrap();
        builder.build_gtk(vbox)?
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
