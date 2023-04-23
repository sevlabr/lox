fn main() {
    scope("str");
}

fn scope(a: &str) {
    let a = "local";
    println!("{}", a);
    
    let b = "outer";
    {
        let b = b;
        println!("{}", b);
    }
    
    let c = "global";
    {
        let show_c = || {
            println!("{}", c);
        };
        
        show_c();
        let c = "block";
        show_c();
    }
}

// Output:
// local
// outer
// global
// global
