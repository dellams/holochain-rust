use crate::{
    context::Context,
    network::entry_with_header::EntryWithHeader,
    workflows::{
        hold_entry::hold_entry_workflow, hold_entry_remove::hold_remove_workflow,
        hold_entry_update::hold_update_workflow, hold_link::hold_link_workflow,
        remove_link::remove_link_workflow,
    },
};
use holochain_core_types::{crud_status::CrudStatus, eav::Attribute};
use holochain_net::connection::json_protocol::{DhtMetaData, EntryData};
use std::{str::FromStr, sync::Arc, thread};

/// The network requests us to store (i.e. hold) the given entry.
pub fn handle_store_entry(dht_data: EntryData, context: Arc<Context>) {
    context.log(format!(
        "debug/net/handle: handle_store_entry: entry data={:?}",
        dht_data
    ));
    let context2 = context.clone();
    serde_json::from_str(
        &serde_json::to_string(&dht_data.entry_content)
            .expect("expected json in dht entry content"),
    )
    .map(|entry_with_header| {
        thread::spawn(move || {
            match context.block_on(hold_entry_workflow(&entry_with_header, context.clone())) {
                Err(error) => context.log(format!("err/net/dht: {}", error)),
                _ => (),
            }
        })
    })
    .map_err(|err| context2.log(format!("err/net/dht: not storing due to: {:?}", err)))
    .ok();
}

/// The network requests us to store meta information (links/CRUD/etc) for an
/// entry that we hold.
pub fn handle_store_meta(dht_meta_data: DhtMetaData, context: Arc<Context>) {
    let attr = dht_meta_data.clone().attribute;
    // @TODO: If network crates will switch to using the `Attribute` enum,
    // we can match on the enum directly
    if attr == Attribute::Link.to_string() {
        context.log("debug/net/handle: HandleStoreMeta: got LINK. processing...");
        // TODO: do a loop on content once links properly implemented
        assert_eq!(dht_meta_data.content_list.len(), 1);
        let entry_with_header: EntryWithHeader = serde_json::from_str(
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHeader"),
        )
        .expect("dht_meta_data should be EntryWithHeader");
        thread::spawn(move || {
            match context.block_on(hold_link_workflow(&entry_with_header, &context.clone())) {
                Err(error) => context.log(format!("err/net/dht: {}", error)),
                _ => (),
            }
        });
    } else if attr == Attribute::LinkRemove.to_string() {
        context.log("debug/net/handle: HandleStoreMeta: got LINK REMOVAL. processing...");
        // TODO: do a loop on content once links properly implemented
        assert_eq!(dht_meta_data.content_list.len(), 1);
        let entry_with_header: EntryWithHeader = serde_json::from_str(
            //should be careful doing slice access, it might panic
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHeader"),
        )
        .expect("dht_meta_data should be EntryWithHeader");
        thread::spawn(move || {
            if let Err(error) =
                context.block_on(remove_link_workflow(&entry_with_header, &context.clone()))
            {
                context.log(format!("err/net/dht: {}", error))
            }
        });
    } else if CrudStatus::from_str(&attr)
        .expect("Could not convert deleted attribute to CrudStatus")
        == CrudStatus::Deleted
    {
        context.log("debug/net/handle: HandleStoreMeta: got CRUD STATUS. processing...");

        let entry_with_header: EntryWithHeader = serde_json::from_str(
            //should be careful doing slice access, it might panic
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHeader"),
        )
        .expect("dht_meta_data should be EntryWithHeader");
        thread::spawn(move || {
            if let Err(error) =
                context.block_on(hold_remove_workflow(entry_with_header, context.clone()))
            {
                context.log(format!("err/net/dht: {}", error))
            }
        });
    } else if CrudStatus::from_str(&attr)
        .expect("Could not convert modified attribute to CrudStatus")
        == CrudStatus::Modified
    {
        context.log("debug/net/handle: HandleStoreMeta: got CRUD LINK. processing...");
        let entry_with_header: EntryWithHeader = serde_json::from_str(
            //should be careful doing slice access, it might panic
            &serde_json::to_string(&dht_meta_data.content_list[0])
                .expect("dht_meta_data should be EntryWithHeader"),
        )
        .expect("dht_meta_data should be EntryWithHeader");
        thread::spawn(move || {
            if let Err(error) =
                context.block_on(hold_update_workflow(entry_with_header, context.clone()))
            {
                context.log(format!("err/net/dht: {}", error))
            }
        });
    }
}
