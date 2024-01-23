use dioxus::prelude::*;
use std::time::Duration;

fn main() {
    // tracing_subscriber::fmt::init();
    launch(app);
}

fn app() -> Element {
    let mut count = use_signal(|| 0);
    let mut counts = use_signal(|| vec![1, 2, 3]);

    use_effect(move || {
        counts.write().resize(count(), 0);
    });

    rsx! {
        h3 { "length: {count}" }
        button { onclick: move |_| count += 1, "Increase length" }
        button { onclick: move |_| count -= 1, "Decrease length" }
        for i in 0..counts.read().len() {
            Child {
                signal: counts,
                idx: i,
            }
        }
    }
}

#[component]
fn Child(signal: Signal<Vec<usize>>, idx: usize) -> Element {
    println!("Running child {idx} ");
    rsx! {
        h2 { "{signal.read()[idx]}" }
    }
}

// fn main() {
//     launch_desktop(app);
// }

// fn app() -> Element {
//     let mut running = use_signal(|| true);
//     let mut count = use_signal(|| 0);
//     let mut saved_values = use_signal(|| vec![0.to_string()]);

//     // Signals can be used in async functions without an explicit clone since they're 'static and Copy
//     // Signals are backed by a runtime that is designed to deeply integrate with Dioxus apps
//     use_future(|| async move {
//         loop {
//             if running() {
//                 count += 1;
//             }

//             let val = running.read();

//             tokio::time::sleep(Duration::from_millis(400)).await;

//             println!("Running: {}", *val);
//         }
//     });

//     rsx! {
//         h1 { "High-Five counter: {count}" }
//         button { onclick: move |_| count += 1, "Up high!" }
//         button { onclick: move |_| count -= 1, "Down low!" }
//         button { onclick: move |_| running.toggle(), "Toggle counter" }
//         button { onclick: move |_| saved_values.push(count.cloned().to_string()), "Save this value" }
//         button { onclick: move |_| saved_values.write().clear(), "Clear saved values" }

//         // We can do boolean operations on the current signal value
//         if count() > 5 {
//             h2 { "High five!" }
//         }

//         // We can cleanly map signals with iterators
//         for value in saved_values.read().iter() {
//             h3 { "Saved value: {value}" }
//         }

//         // We can also use the signal value as a slice
//         if let [ref first, .., ref last] = saved_values.read().as_slice() {
//             li { "First and last: {first}, {last}" }
//         } else {
//             "No saved values"
//         }
//     }
// }
