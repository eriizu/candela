# in zshrc need: fpath=(~/.config/zsh/completions $fpath)

mkdir -pv ~/.config/zsh/completions/
cargo run -- completions zsh > ~/.config/zsh/completions/_candela
