use chat::{channel::Channel, Path};

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
// use chat::ChannelInput;
use proptest::{
    prelude::prop,
    prop_compose, proptest,
    strategy::{Just, Strategy},
};

prop_compose! {
    fn arb_timestamp()(year in 0..3i32, month in 0..3u32, day in 0..3u32, hour in 0..3u32, seconds in 0..3u32) -> hdk::prelude::Timestamp {
        let datetime = NaiveDateTime::new(NaiveDate::from_ymd(year + 1970, month, day), NaiveTime::from_hms(hour, 0, seconds));
        let utc = DateTime::from_utc(datetime, Utc);
        let timestamp: hdk::prelude::Timestamp = utc.into();
        timestamp
    }
}

prop_compose! {
    fn arb_path()(timestamp in arb_timestamp()) -> Path {
        let channel = Channel {
            category: "General".into(),
            uuid: "uuid".into(),
        };
        let channel_path = Path::from(channel);
        chat::pagination_helper::timestamp_into_path(channel_path, timestamp).unwrap()
    }
}

proptest! {
    #[test]
    fn test((message_timestamps, earliest_seen_idx, target_count) in prop::collection::vec(arb_timestamp(), 1..10).prop_flat_map(|vec| {
        let len = vec.len();
        (Just(vec), 0..len, 0..len + 1)
    })) {
        let _mock_hdk = hdk::prelude::MockHdkT::new();

        let channel = Channel {
            category: "General".into(),
            uuid: "uuid".into(),
        };
        let channel_path = Path::from(channel);

        todo!()
    }
}
