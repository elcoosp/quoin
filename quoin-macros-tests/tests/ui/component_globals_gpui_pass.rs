use quoin::component;

#[derive(Clone)]
pub struct RouterState {
    pub route: String,
}

component! {
    pub GlobalsTest {
        globals {
            router: RouterState,
        }
        state {
            count: u32 = 0,
        }
        render {
            let _ = count.get();
            "hello"
        }
    }
}

fn main() {}
