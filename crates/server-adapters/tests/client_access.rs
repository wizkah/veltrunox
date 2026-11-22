//! The client core signs in, sees its rights and administers users against a real server.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::sync::{Arc, mpsc};
use support::{WAIT, secured_server};
use veltrunox_client_adapters::{HttpServerLink, SystemTimer, open_channel};
use veltrunox_client_app::{
    ChannelAccess, ChannelEvent, Credentials, LinkError, ManagedUser, NewUser, Rights, ServerEvent,
    UserChange, add_user, delete_user, edit_user, load_access_status, load_library, load_rights,
    load_users, set_guest_access, set_password_policy, set_up_first_admin, sign_in, sign_out,
};
use veltrunox_core::{Capability, Millis, PasswordPolicy, Role};

const PASSWORD: &str = "correct horse battery";

fn credentials(name: &str, password: &str) -> Credentials {
    Credentials {
        user_name: name.to_owned(),
        password: password.to_owned(),
    }
}

fn link(url: &str, token: Option<&str>) -> HttpServerLink {
    HttpServerLink::with_token(url, token.map(str::to_owned))
}

/// Opens the channel and waits for what the server says about the sign-in.
fn rights_on_channel(url: &str, access: ChannelAccess) -> Option<Rights> {
    let (sender, events) = mpsc::channel();
    let _channel = open_channel(
        url,
        access,
        Arc::new(SystemTimer),
        || Millis::new(0),
        move |event| {
            let _ = sender.send(event);
        },
    );
    loop {
        match events.recv_timeout(WAIT).ok()? {
            ChannelEvent::Server(ServerEvent::Authenticated(rights)) => return Some(rights),
            ChannelEvent::Server(ServerEvent::Rejected(_)) => return None,
            _ => {}
        }
    }
}

#[tokio::test]
async fn setup_sign_in_and_sign_out_follow_the_server_rules() {
    let url = secured_server().await;
    let anonymous = link(&url, None);
    let status = load_access_status(&anonymous).await.unwrap();
    assert!(status.setup_needed);
    assert!(!status.guest_enabled);

    let admin = set_up_first_admin(&anonymous, &credentials("root", PASSWORD))
        .await
        .unwrap();
    assert_eq!(admin.rights.role, Role::Admin);
    assert!(admin.rights.allows(Capability::ManageUsers));
    assert!(matches!(
        set_up_first_admin(&anonymous, &credentials("again", PASSWORD)).await,
        Err(LinkError::Refused(_))
    ));

    assert!(matches!(
        sign_in(&anonymous, &credentials("root", "not the password")).await,
        Err(LinkError::Unauthorized)
    ));
    let again = sign_in(&anonymous, &credentials("ROOT", PASSWORD))
        .await
        .unwrap();
    let signed = link(&url, Some(&again.token));
    assert!(load_library(&signed).await.is_ok());
    assert!(matches!(
        load_library(&anonymous).await,
        Err(LinkError::Unauthorized)
    ));

    sign_out(&signed).await.unwrap();
    assert!(matches!(
        load_library(&signed).await,
        Err(LinkError::Unauthorized)
    ));
}

#[tokio::test]
async fn too_many_failed_sign_ins_report_the_wait() {
    let url = secured_server().await;
    let anonymous = link(&url, None);
    set_up_first_admin(&anonymous, &credentials("root", PASSWORD))
        .await
        .unwrap();
    for _ in 0..4 {
        let _ = sign_in(&anonymous, &credentials("root", "wrong wrong wrong")).await;
    }
    assert!(matches!(
        sign_in(&anonymous, &credentials("root", PASSWORD)).await,
        Err(LinkError::TooManyAttempts { .. })
    ));
}

#[tokio::test]
async fn the_admin_manages_users_and_a_user_is_refused() {
    let url = secured_server().await;
    let anonymous = link(&url, None);
    let admin = set_up_first_admin(&anonymous, &credentials("root", PASSWORD))
        .await
        .unwrap();
    let admin_link = link(&url, Some(&admin.token));

    let created: ManagedUser = add_user(
        &admin_link,
        &NewUser {
            user_name: "ann".to_owned(),
            password: PASSWORD.to_owned(),
            role: Role::User,
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        add_user(
            &admin_link,
            &NewUser {
                user_name: "Ann".to_owned(),
                password: PASSWORD.to_owned(),
                role: Role::User,
            },
        )
        .await,
        Err(LinkError::Refused(_))
    ));
    assert_eq!(load_users(&admin_link).await.unwrap().len(), 2);

    let ann = sign_in(&anonymous, &credentials("ann", PASSWORD))
        .await
        .unwrap();
    assert!(!ann.rights.allows(Capability::ManageUsers));
    assert!(ann.rights.allows(Capability::Upload));
    let ann_link = link(&url, Some(&ann.token));
    assert!(matches!(
        load_users(&ann_link).await,
        Err(LinkError::Forbidden)
    ));
    assert!(matches!(
        set_guest_access(&ann_link, true).await,
        Err(LinkError::Forbidden)
    ));

    edit_user(
        &admin_link,
        created.id,
        &UserChange {
            role: Some(Role::Admin),
            ..UserChange::default()
        },
    )
    .await
    .unwrap();
    assert!(load_users(&ann_link).await.is_ok());
    delete_user(&admin_link, created.id).await.unwrap();
    assert!(matches!(
        load_rights(&ann_link).await,
        Err(LinkError::Unauthorized)
    ));
}

#[tokio::test]
async fn guest_access_follows_the_admin_switch() {
    let url = secured_server().await;
    let anonymous = link(&url, None);
    let admin = set_up_first_admin(&anonymous, &credentials("root", PASSWORD))
        .await
        .unwrap();
    let admin_link = link(&url, Some(&admin.token));
    assert!(matches!(
        load_rights(&anonymous).await,
        Err(LinkError::Unauthorized)
    ));

    set_guest_access(&admin_link, true).await.unwrap();
    assert!(load_access_status(&anonymous).await.unwrap().guest_enabled);
    let rights = load_rights(&anonymous).await.unwrap();
    assert_eq!(rights.role, Role::Guest);
    assert!(rights.allows(Capability::PlayControl));
    assert!(!rights.allows(Capability::ManageUsers));
}

#[tokio::test]
async fn the_password_rules_follow_the_admin_choice() {
    let url = secured_server().await;
    let anonymous = link(&url, None);
    let admin = set_up_first_admin(&anonymous, &credentials("root", PASSWORD))
        .await
        .unwrap();
    let admin_link = link(&url, Some(&admin.token));
    assert_eq!(
        load_access_status(&anonymous)
            .await
            .unwrap()
            .password_policy,
        PasswordPolicy::Standard
    );

    set_password_policy(&admin_link, PasswordPolicy::Strict)
        .await
        .unwrap();
    assert_eq!(
        load_access_status(&anonymous)
            .await
            .unwrap()
            .password_policy,
        PasswordPolicy::Strict
    );
    // Without the right, the change is refused.
    assert!(
        set_password_policy(&anonymous, PasswordPolicy::Standard)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn the_channel_reports_the_rights_or_a_refusal() {
    let url = secured_server().await;
    let anonymous = link(&url, None);
    let admin = set_up_first_admin(&anonymous, &credentials("root", PASSWORD))
        .await
        .unwrap();

    let (url_for_channel, token) = (url.clone(), admin.token.clone());
    let signed = tokio::task::spawn_blocking(move || {
        rights_on_channel(&url_for_channel, ChannelAccess::Token(token))
    })
    .await
    .unwrap();
    assert_eq!(signed.unwrap().role, Role::Admin);

    let url_for_channel = url.clone();
    let refused = tokio::task::spawn_blocking(move || {
        rights_on_channel(&url_for_channel, ChannelAccess::Token("bogus".to_owned()))
    })
    .await
    .unwrap();
    assert_eq!(refused, None);

    set_guest_access(&link(&url, Some(&admin.token)), true)
        .await
        .unwrap();
    let guest = tokio::task::spawn_blocking(move || rights_on_channel(&url, ChannelAccess::Guest))
        .await
        .unwrap();
    assert_eq!(guest.unwrap().role, Role::Guest);
}
