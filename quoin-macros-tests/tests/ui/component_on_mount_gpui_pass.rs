use quoin::component;

component! {
    pub OnMountTest {
        state {
            mounted: bool = false,
        }
        on_mount {
            mounted.set(true);
        }
        render {
            "hello"
        }
    }
}

fn main() {}
