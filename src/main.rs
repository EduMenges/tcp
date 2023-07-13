pub mod text_to_midi;

fn main() {
    let caractere = '5';

    match caractere {
        c if c.is_digit(10) => {
            println!("O caractere {} é um dígito.", c);
        }
        _ => {
            println!("O caractere {} não é um dígito.", caractere);
        }
    }
}