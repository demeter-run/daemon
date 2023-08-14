use kube::CustomResourceExt;
use std::io::stdout;

fn main() {
    let stdout = stdout();

    serde_yaml::to_writer(stdout, &demeter_operator::authtokens::AuthToken::crd()).unwrap();
}
