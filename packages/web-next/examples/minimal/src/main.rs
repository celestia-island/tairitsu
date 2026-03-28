use tairitsu_web_next::*;

fn main() {
    // Initialize the web platform
    init();

    // Check if we're in browser or SSR mode
    #[cfg(feature = "ssr")]
    {
        println!("Running in SSR mode");
        assert!(is_ssr());
    }

    #[cfg(feature = "browser")]
    {
        println!("Running in browser mode");
        assert!(is_browser());
    }

    // Test VDOM features if enabled
    #[cfg(feature = "vdom")]
    {
        use tairitsu_vdom::VNode;
        let vnode = VNode::Text("Hello, Web-Next!".into());
        println!("VDOM test: {}", vnode.text().unwrap());
    }

    // Test hooks if enabled
    #[cfg(feature = "hooks")]
    {
        use tairitsu_hooks::use_signal;
        let signal = use_signal(|| 42);
        println!("Signal test: {}", *signal.get());
    }

    println!("Web-Next package initialized successfully!");
}