use super::{Folder, Message, Thread};

pub(super) fn mailbox() -> Vec<Thread> {
    let mut first = thread(
        "Dispatch dashboard review",
        "Work",
        true,
        true,
        Folder::Inbox,
        (
            "Casey Imani",
            "casey@spenstar.example",
            "Nov 25, 9:42 AM",
            "Hi Alex,\n\nThe updated dispatch dashboard is ready for a final review. Carrier exceptions are grouped by hub, and the weekly view has a clearer breakdown of outbound volume.\n\nCould you take a look at the shipment filters and the mobile layout before Thursday’s handoff?\n\nThanks,\nCasey",
        ),
    );
    first.messages.push(message(
        "Morgan Ortiz", "morgan@spenstar.example", "Nov 25, 11:18 AM",
        "I reviewed the warehouse filters this morning. Everything looks good from the dispatch side.\n\nI added two notes about date ranges to the handoff checklist. Alex, can you confirm which month should be selected by default?\n\nMorgan",
    ));
    vec![
        first,
        thread(
            "Warehouse walkthrough",
            "Travel",
            false,
            true,
            Folder::Inbox,
            (
                "Morgan Ortiz",
                "morgan@spenstar.example",
                "Nov 25, 8:10 AM",
                "The walkthrough starts at 10 AM. Meet the North team at the main entrance.",
            ),
        ),
        thread(
            "November carrier invoice",
            "Billing",
            true,
            true,
            Folder::Inbox,
            (
                "Elliot Park",
                "elliot@spenstar.example",
                "Nov 24, 4:22 PM",
                "The November invoice is ready for reconciliation. The fuel surcharge is itemized.",
            ),
        ),
        thread(
            "New delivery exception",
            "Urgent",
            false,
            true,
            Folder::Inbox,
            (
                "Jordan Kim",
                "jordan@spenstar.example",
                "Nov 24, 1:05 PM",
                "West warehouse flagged a temperature exception for shipment SHP-0480.",
            ),
        ),
        thread(
            "Coffee after standup?",
            "Personal",
            false,
            false,
            Folder::Inbox,
            (
                "Avery Chen",
                "avery@spenstar.example",
                "Nov 23, 9:45 AM",
                "I’ll be at the café downstairs after the operations standup.",
            ),
        ),
        thread(
            "West hub weekly brief",
            "Work",
            false,
            false,
            Folder::Inbox,
            (
                "Leila Diaz",
                "leila@spenstar.example",
                "Nov 21, 2:20 PM",
                "West completed its final inventory count on Friday.",
            ),
        ),
        thread(
            "Parking for the site visit",
            "Travel",
            false,
            false,
            Folder::Inbox,
            (
                "Noah Reed",
                "noah@spenstar.example",
                "Nov 20, 8:02 AM",
                "Visitor spots are available along the south entrance.",
            ),
        ),
        thread(
            "October credit adjustment",
            "Billing",
            false,
            false,
            Folder::Inbox,
            (
                "Elliot Park",
                "elliot@spenstar.example",
                "Nov 18, 10:15 AM",
                "The adjustment has posted to the carrier account.",
            ),
        ),
        thread(
            "October dispatch report",
            "Work",
            false,
            false,
            Folder::Archive,
            (
                "Casey Imani",
                "casey@spenstar.example",
                "Nov 1, 10:00 AM",
                "The October summary is filed with the rest of the operational reports.",
            ),
        ),
    ]
}

fn message(sender: &str, email: &str, time: &str, body: &str) -> Message {
    Message {
        sender: sender.into(),
        email: email.into(),
        time: time.into(),
        body: body.into(),
    }
}

fn thread(
    subject: &str,
    label: &'static str,
    starred: bool,
    unread: bool,
    folder: Folder,
    from: (&str, &str, &str, &str),
) -> Thread {
    Thread {
        subject: subject.into(),
        label,
        starred,
        unread,
        folder,
        messages: vec![message(from.0, from.1, from.2, from.3)],
    }
}
