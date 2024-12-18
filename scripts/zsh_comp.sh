# in zshrc need: fpath=(~/.config/zsh/completions $fpath)

mkdir -pv ~/.config/zsh/completions/
cargo run --bin candela -- completions zsh > ~/.config/zsh/completions/_candela
