use anyhow::bail;
use serde_json::Value;
use serenity::client::{Context, EventHandler};
use serenity::model::interactions::{Interaction, InteractionData};
use serenity::utils::Colour;
use uuid::Uuid;

use crate::entities::{Content, User};
use crate::handlers::Handler;

pub struct Conductor {
    pub handler: Handler,
}

pub enum Command {
    UserRegister,
    UserRead,
    UserUpdate(Option<bool>, Option<bool>),
    Bookmark(Uuid),
    UserDelete,
    ContentPost(String),
    ContentRead(Uuid),
    ContentUpdate(Uuid, String),
    Like(Uuid),
    Pin(Uuid),
    ContentDelete(Uuid),
}

pub struct Response {
    title: String,
    rgb: (u8, u8, u8),
    description: String,
    fields: Vec<(String, String)>,
}

impl Conductor {
    pub async fn parse(&self, interaction: &Interaction) -> anyhow::Result<Command> {
        let data = match match interaction.data {
            Some(ref d) => d,
            None => bail!("cannot get interactiion_data."),
        } {
            InteractionData::ApplicationCommand(d) => d,
            _ => bail!(
                "cannot get application_command_interaction_data. (maybe it's message-component? \
                 sorry, not supported.)"
            ),
        };

        let com = match data.name.as_str() {
            "register" => Command::UserRegister,
            "info" => Command::UserRead,
            "change" => {
                let mut admin_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "admin" {
                        false => None,
                        true => match v.value {
                            Some(Value::Bool(ref b)) => Some(Some(b)),
                            _ => Some(None),
                        },
                    })
                    .collect::<Vec<_>>();
                if admin_opt.len() != 1 {
                    bail!("cannot get value: `admin`");
                }
                let admin = admin_opt.remove(0);

                let mut sub_admin_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "sub_admin" {
                        false => None,
                        true => match v.value {
                            Some(Value::Bool(ref b)) => Some(Some(b)),
                            _ => Some(None),
                        },
                    })
                    .collect::<Vec<_>>();
                if admin_opt.len() != 1 {
                    bail!("cannot get value: `sub_admin`");
                }
                let sub_admin = sub_admin_opt.remove(0);

                Command::UserUpdate(admin.copied(), sub_admin.copied())
            },
            "bookmark" => {
                let mut id_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "id" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if id_opt.len() != 1 {
                    bail!("cannot get value: `id`")
                }
                let id = id_opt.remove(0);

                Command::Bookmark(Uuid::parse_str(id.as_str())?)
            },
            "delete_me" => Command::UserDelete,
            "post" => {
                let mut content_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "content" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if content_opt.len() != 1 {
                    bail!("cannot get value: `content`")
                }

                let content = content_opt.remove(0);

                Command::ContentPost(content.clone())
            },
            "get" => {
                let mut id_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "id" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if id_opt.len() != 1 {
                    bail!("cannot get value: `id`")
                }
                let id = id_opt.remove(0);

                Command::ContentRead(Uuid::parse_str(id.as_str())?)
            },
            "edit" => {
                let mut content_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "content" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if content_opt.len() != 1 {
                    bail!("cannot get value: `content`")
                }
                let content = content_opt.remove(0);

                let mut id_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "id" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if id_opt.len() != 1 {
                    bail!("cannot get value: `id`")
                }
                let id = id_opt.remove(0);

                Command::ContentUpdate(Uuid::parse_str(id.as_str())?, content.clone())
            },
            "like" => {
                let mut id_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "id" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if id_opt.len() != 1 {
                    bail!("cannot get value: `id`")
                }
                let id = id_opt.remove(0);

                Command::Like(Uuid::parse_str(id.as_str())?)
            },
            "pin" => {
                let mut id_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "id" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if id_opt.len() != 1 {
                    bail!("cannot get value: `id`")
                }
                let id = id_opt.remove(0);

                Command::Pin(Uuid::parse_str(id.as_str())?)
            },
            "remove" => {
                let mut id_opt = data
                    .options
                    .iter()
                    .filter_map(|v| match v.name == "id" {
                        false => None,
                        true => match v.value {
                            Some(Value::String(ref s)) => Some(s),
                            _ => None,
                        },
                    })
                    .collect::<Vec<_>>();
                if id_opt.len() != 1 {
                    bail!("cannot get value: `id`")
                }
                let id = id_opt.remove(0);

                Command::ContentDelete(Uuid::parse_str(id.as_str())?)
            },
            _ => bail!("unrecognized application_command name."),
        };

        Ok(com)
    }

    pub async fn handle(&self, interaction: &Interaction) -> Response {
        let res: anyhow::Result<Response> = try {
            let user = match interaction.user {
                Some(ref u) => Ok(u),
                None => Err(anyhow::anyhow!(
                    "cannot get user info. (maybe it's DM? sorry, not supported.)"
                )),
            }?;

            let resp: Response = match self.parse(interaction).await? {
                Command::UserRegister => {
                    let User {
                        id,
                        admin,
                        sub_admin,
                        posted,
                        bookmark,
                    } = self.handler.create_user(user.id).await?;

                    Response {
                        title: format!("name: [unimplemented]"), // FIXME
                        rgb: (0, 0, 0),                          // FIXME: 色決め
                        description: format!("id: {}", id),
                        fields: vec![
                            ("is_admin?".to_string(), format!("{}", admin)),
                            ("is_sub_admin?".to_string(), format!("{}", sub_admin)),
                            ("posted:".to_string(), format!("{}", posted.len())),
                            ("bookmarked:".to_string(), format!("{}", bookmark.len())),
                        ],
                    }
                },
                Command::UserRead => {
                    let User {
                        id,
                        admin,
                        sub_admin,
                        posted,
                        bookmark,
                    } = self.handler.read_user(user.id).await?;

                    Response {
                        title: format!("name: [unimplemented]"), // FIXME
                        rgb: (0, 0, 0),                          // FIXME: 色決め
                        description: format!("id: {}", id),
                        fields: vec![
                            ("is_admin?".to_string(), format!("{}", admin)),
                            ("is_sub_admin?".to_string(), format!("{}", sub_admin)),
                            ("posted:".to_string(), format!("{}", posted.len())),
                            ("bookmarked:".to_string(), format!("{}", bookmark.len())),
                        ],
                    }
                },
                Command::UserUpdate(new_admin, new_sub_admin) => {
                    let User {
                        id,
                        admin,
                        sub_admin,
                        posted,
                        bookmark,
                    } = self
                        .handler
                        .update_user(user.id, new_admin, new_sub_admin)
                        .await?;

                    Response {
                        title: format!("name: [unimplemented]"), // FIXME
                        rgb: (0, 0, 0),                          // FIXME: 色決め
                        description: format!("id: {}", id),
                        fields: vec![
                            ("is_admin?".to_string(), format!("{}", admin)),
                            ("is_sub_admin?".to_string(), format!("{}", sub_admin)),
                            ("posted:".to_string(), format!("{}", posted.len())),
                            ("bookmarked:".to_string(), format!("{}", bookmark.len())),
                        ],
                    }
                },
                Command::Bookmark(id) => {
                    let () = self.handler.bookmark_update_user(user.id, id).await?;
                    let Content {
                        id,
                        content,
                        liked,
                        bookmarked,
                        pinned,
                    } = self.handler.read_content(id).await?;

                    Response {
                        title: "bookmarked".to_string(),
                        rgb: (0, 0, 0), // TODO: 色決め
                        description: format!("from user (id): {}", user.id),
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("content:".to_string(), content),
                            ("liked:".to_string(), format!("{}", liked.len())),
                            ("pinned:".to_string(), format!("{}", pinned.len())),
                            ("bookmarked:".to_string(), format!("{}", bookmarked)),
                        ],
                    }
                },
                Command::UserDelete => {
                    let () = self.handler.delete_user(user.id).await?;

                    Response {
                        title: "user deleted".to_string(),
                        rgb: (0, 0, 0), // TODO: 色決め
                        description: "see you!".to_string(),
                        fields: vec![],
                    }
                },
                Command::ContentPost(content) => {
                    let Content {
                        id,
                        content,
                        liked,
                        bookmarked,
                        pinned,
                    } = self
                        .handler
                        .create_content_and_posted_update_user(content, user.id)
                        .await?;

                    Response {
                        title: "posted".to_string(),
                        rgb: (0, 0, 0), // TODO: 色決め
                        description: format!("from user (id): {}", user.id),
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("content:".to_string(), content),
                            ("liked:".to_string(), format!("{}", liked.len())),
                            ("pinned:".to_string(), format!("{}", pinned.len())),
                            ("bookmarked:".to_string(), format!("{}", bookmarked)),
                        ],
                    }
                },
                Command::ContentRead(id) => {
                    let Content {
                        id,
                        content,
                        liked,
                        bookmarked,
                        pinned,
                    } = self.handler.read_content(id).await?;

                    Response {
                        title: "showing content".to_string(),
                        rgb: (0, 0, 0), // TODO: 色決め
                        description: "".to_string(),
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("content:".to_string(), content),
                            ("liked:".to_string(), format!("{}", liked.len())),
                            ("pinned:".to_string(), format!("{}", pinned.len())),
                            ("bookmarked:".to_string(), format!("{}", bookmarked)),
                        ],
                    }
                },
                Command::ContentUpdate(id, new_content) => {
                    let Content {
                        id,
                        content,
                        liked,
                        pinned,
                        bookmarked,
                    } = self.handler.update_content(id, new_content).await?;

                    Response {
                        title: "updated".to_string(),
                        rgb: (0, 0, 0), // TODO: 色決め
                        description: format!("from user (id): {}", user.id),
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("content:".to_string(), content),
                            ("liked:".to_string(), format!("{}", liked.len())),
                            ("pinned:".to_string(), format!("{}", pinned.len())),
                            ("bookmarked:".to_string(), format!("{}", bookmarked)),
                        ],
                    }
                },
                Command::Like(id) => {
                    let () = self.handler.like_update_content(id, user.id).await?;

                    Response {
                        title: "liked".to_string(),
                        description: format!("from user (id): {}", user.id),
                        rgb: (0, 0, 0), // TODO: 色決め
                        fields: vec![("id:".to_string(), format!("{}", id))],
                    }
                },
                Command::Pin(id) => {
                    let () = self.handler.pin_update_content(id, user.id).await?;

                    Response {
                        title: "pinned".to_string(),
                        description: format!("from user (id): {}", user.id),
                        rgb: (0, 0, 0), // TODO: 色決め
                        fields: vec![("id:".to_string(), format!("{}", id))],
                    }
                },
                Command::ContentDelete(id) => {
                    let () = self.handler.delete_content(id).await?;

                    Response {
                        title: "deleted".to_string(),
                        description: "i'm sad...".to_string(),
                        rgb: (0, 0, 0), // TODO: 色決め
                        fields: vec![("id:".to_string(), format!("{}", id))],
                    }
                },
            };

            resp
        };

        match res {
            Ok(r) => r,
            Err(e) => Response {
                title: "error occurred".to_string(),
                rgb: (Colour::RED.r(), Colour::RED.g(), Colour::RED.b()),
                description: format!("{}", e),
                fields: vec![],
            },
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Conductor {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Response {
            title,
            rgb,
            description,
            mut fields,
        } = self.handle(&interaction).await;
        let (r, g, b) = rgb;

        let res = interaction
            .create_interaction_response(ctx.http, |cir| {
                cir.interaction_response_data(|cird| {
                    cird.create_embed(|ce| {
                        ce.title(title)
                            .colour(Colour::from_rgb(r, g, b))
                            .description(description)
                            .fields(
                                fields
                                    .drain(..)
                                    .map(|(s1, s2)| (s1, s2, false))
                                    .collect::<Vec<_>>(),
                            )
                    })
                })
            })
            .await;

        match res {
            Ok(o) => o,
            Err(e) => eprintln!("{}", e),
        };
    }
}
