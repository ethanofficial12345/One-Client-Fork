use freya::prelude::*;
use freya::query::{MutationCapability, MutationStateData, UseMutation};
use oneclient_auth::{MinecraftAccount, validate_offline_username};

use crate::components::{Avatar, Button, Icon, IconType, TextInput, use_microsoft_login};
use crate::hooks::{
    AddOfflineAccountKeys, try_default_account, use_add_offline_account, use_current_account,
};
use crate::routes::Route;
use crate::theme::colors;
use crate::view::onboarding::{
    onboarding_illustration, onboarding_nav, onboarding_page, step_heading,
};

#[derive(PartialEq)]
pub struct OnboardingAccount;

impl Component for OnboardingAccount {
    fn render(&self) -> impl IntoElement {
        let account_query = use_current_account();
        let msa = use_microsoft_login();
        let add_offline = use_add_offline_account();

        let mut offline_name = use_state(String::new);

        let account = try_default_account(&account_query);
        let has_account = account.is_some();

        // Validate as the user types so the button reflects whether the name
        // can become an identity, and the same rule runs again in the service
        let typed_name = offline_name.read().trim().to_string();
        let validation_error = (!typed_name.is_empty())
            .then(|| validate_offline_username(&typed_name).err())
            .flatten()
            .map(|err| err.to_string());
        let offline_error = validation_error
            .clone()
            .or_else(|| mutation_err_text(&add_offline));
        let can_submit = !typed_name.is_empty() && validation_error.is_none();

        let on_add_offline = move |_| {
            let name = offline_name.peek().trim().to_string();
            if name.is_empty() || validate_offline_username(&name).is_err() {
                return;
            }
            add_offline.mutate(AddOfflineAccountKeys { username: name });
        };

        let content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(24.)
            .child(step_heading(
                "Account",
                "Before you continue, we require you to own a copy of Minecraft: Java Edition.",
            ))
            .child(match &account {
                Some(account) => account_preview(account).into_element(),
                None => {
                    let start = msa.clone();
                    rect()
                        .vertical()
                        .width(Size::fill())
                        .spacing(16.)
                        .child(sign_in_card(msa.pending, msa.error.clone(), move |_| {
                            start.start()
                        }))
                        .child(offline_card(
                            offline_name,
                            offline_error,
                            can_submit,
                            on_add_offline,
                        ))
                        .into_element()
                }
            })
            .into_element();

        let page = onboarding_page(
            onboarding_illustration(IconType::OnboardingAccount),
            content,
            onboarding_nav(
                Some(Route::OnboardingLanguage {}),
                Route::OnboardingBundles {},
                has_account,
            ),
        );

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .child(page)
            .maybe_child(msa.popup())
    }
}

fn account_preview(account: &MinecraftAccount) -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .spacing(24.)
        .child(
            rect()
                .horizontal()
                .spacing(12.)
                .cross_align(Alignment::Center)
                .child(
                    Avatar::new(account.id.to_string())
                        .width(Size::px(48.))
                        .height(Size::px(48.)),
                )
                .child(
                    rect()
                        .vertical()
                        .spacing(4.)
                        .child(
                            label()
                                .text(account.username.clone())
                                .font_size(16.)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .color(colors::fg_primary()),
                        )
                        .child(
                            label()
                                .text(format!(
                                    "{} · {}",
                                    if account.is_offline() {
                                        "Offline"
                                    } else {
                                        "Microsoft"
                                    },
                                    account.id
                                ))
                                .font_size(12.)
                                .color(colors::fg_secondary()),
                        ),
                ),
        )
        .into_element()
}

fn offline_card(
    username: State<String>,
    error: Option<String>,
    enabled: bool,
    on_add: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    rect()
        .vertical()
        .width(Size::fill())
        .spacing(8.)
        .child(
            label()
                .text("No Microsoft account? Play offline with a local username.")
                .font_size(13.)
                .color(colors::fg_secondary()),
        )
        .child(
            rect()
                .horizontal()
                .width(Size::fill())
                .spacing(8.)
                .child(
                    TextInput::new(username)
                        .placeholder("Offline username")
                        .width(Size::flex(1.)),
                )
                .child(
                    Button::new()
                        .secondary()
                        .enabled(enabled)
                        .on_press(on_add)
                        .child(Icon::new(IconType::Globe01).size(16.))
                        .text("Play offline"),
                ),
        )
        .maybe_child(error.map(|message| {
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(6.)
                .child(
                    Icon::new(IconType::AlertTriangle)
                        .size(13.)
                        .color(colors::danger()),
                )
                .child(label().text(message).font_size(12.).color(colors::danger()))
                .into_element()
        }))
        .into_element()
}

fn sign_in_card(
    pending: bool,
    error: Option<String>,
    on_add: impl FnMut(Event<PressEventData>) + 'static,
) -> impl IntoElement {
    rect()
        .vertical()
        .spacing(12.)
        .cross_align(Alignment::Start)
        .child(
            Button::new()
                .primary()
                .large()
                .enabled(!pending)
                .on_press(on_add)
                .child(Icon::new(IconType::Globe01).size(16.))
                .text(if pending {
                    "Signing in..."
                } else {
                    "Add Account"
                }),
        )
        .maybe_child(error.map(|message| {
            rect()
                .horizontal()
                .cross_align(Alignment::Center)
                .spacing(6.)
                .child(
                    Icon::new(IconType::AlertTriangle)
                        .size(13.)
                        .color(colors::danger()),
                )
                .child(label().text(message).font_size(12.).color(colors::danger()))
                .into_element()
        }))
        .into_element()
}

fn mutation_err_text<M>(mutation: &UseMutation<M>) -> Option<String>
where
    M: MutationCapability,
    M::Err: std::fmt::Display,
{
    match &*mutation.read().state() {
        MutationStateData::Settled { res: Err(err), .. } => Some(err.to_string()),
        MutationStateData::Loading {
            res: Some(Err(err)),
        } => Some(err.to_string()),
        _ => None,
    }
}
