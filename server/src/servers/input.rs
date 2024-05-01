

use std::process;
use std::str::{FromStr};
use actix::{Actor, Addr};
use tokio::{io};
use tokio::io::{AsyncBufReadExt};
use anyhow::{Result};
use strum::{Display, EnumIter, IntoEnumIterator};
use crate::servers::authentication::{AuthenticationServer, NewAdminAccessToken};
use crate::servers::input::Commands::HELP;

#[derive(Debug, PartialEq, EnumIter, Display)]
enum Commands{
    STOP,
    ADMINCODE,
    PERMIT(String),
    HELP,
}


impl Commands {

    fn to_help(self) -> &'static str {
        match self {
            Commands::STOP => "Stopping the server",
            Commands::ADMINCODE => "Generate a Admin Access Code",
            Commands::PERMIT(_) => "Give the the input Discord id admin right",
            Commands::HELP => "Show the helping information",
        }


    }



    async fn run(self, game_server: &Addr<AuthenticationServer>) {
        match self {
            Commands::STOP =>  {
                process::exit(0);
            },
            Commands::ADMINCODE => {
                println!("Request for and Admin code");
                let request_token = game_server.send(NewAdminAccessToken);
                match request_token.await {
                    Ok(admin_token) => {
                        println!("Your Admin access Token is [{}] its ", admin_token.token)
                    }
                    Err(error) => {
                        println!("There was something wrong by creating the Admin access Token")
                    }
                }

            },
            Commands::PERMIT(discord_id) => {
                println!("ID:{}",discord_id);
            },
            Commands::HELP => {
                for command in Commands::iter() {
                    println!("/{} : {}",command.to_string(), command.to_help())
                }
            }
        }
    }


}

impl FromStr for Commands {

    type Err = ();

    fn from_str(input: &str) -> Result<Commands, ()> {
        let mut parts = input.split_whitespace();
        let first_word = parts.next();
        let first_word = match first_word {
            None => return Err(()),
            Some(word ) => word.to_uppercase()
        };






        match first_word.as_str() {
            "/STOP"  => Ok(Commands::STOP),
            "/ADMINCODE"  => Ok(Commands::ADMINCODE),
            "/PERMIT" => {
                let second_word = parts.next();
                let second_word = match second_word {
                    None => return return Err(()),
                    Some(word ) => word
                };
                Ok(Commands::PERMIT(second_word.into()))
            },
            "/HELP" => Ok(HELP),
            _input => return Err(()),
        }
    }
}

#[derive(Debug)]
pub struct InputServer {
    game_server: Addr<AuthenticationServer>
}

impl InputServer {
    pub fn init(game_server: Addr<AuthenticationServer>) -> Self {
        InputServer {
            game_server,
        }
    }
    //FIXME BUG NOT PRINT! WIlL ONLY BE SEE AFTER TYPING
    pub(crate) async fn read_input(self) -> Result<()> {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin);
        println!("Server started. Enter '/stop' to stop the server.");

        // Spawn a separate task to handle printing lines while waiting for input


        loop {
            let mut input = String::new();
            reader.read_line(&mut input).await.expect("Can´t read command line");

            let command = Commands::from_str(input.as_str());
            match command {
                Ok(command) => {
                    command.run(&self.game_server).await;
                }
                Err(_) => {
                    println!("No command found for: {}", input);
                }
            }
        }


        Ok(())
    }


}





impl Actor for InputServer {
    type Context = actix::Context<Self>;

    fn start(self) -> Addr<Self> where Self: Actor<Context = actix::Context<Self>> {
        actix::Context::new().run(self)
    }
}