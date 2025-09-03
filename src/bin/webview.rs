use clap::Parser;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    platform::unix::EventLoopBuilderExtUnix,
    window::WindowBuilder,
};
use webapps::WebviewArgs;
use wry::{WebContext, WebViewBuilder};

fn main() -> wry::Result<()> {
    let args = WebviewArgs::parse();

    gtk::init().unwrap();

    gtk::glib::set_program_name(args.window_title.clone().into());
    gtk::glib::set_application_name(&args.app_id);

    tracing::debug!("Starting webview with args: {:?}", args);

    let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

    let window = WindowBuilder::new()
        .with_title(args.window_title)
        .build(&event_loop)
        .unwrap();

    let mut context = WebContext::new(args.profile);

    let builder = WebViewBuilder::new_with_web_context(&mut context)
        .with_url(args.url)
        .with_new_window_req_handler(|url, features| {
            println!("new window req: {url} {features:?}");
            wry::NewWindowResponse::Allow
        });

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
