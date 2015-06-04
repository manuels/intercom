use dbus::MessageItem;
use std::borrow::Cow;

pub trait ConvertDbusItem {
	fn from_dbus_item(item: &MessageItem) -> Option<Self>;
	fn to_dbus_item(&self) -> MessageItem;
}

impl ConvertDbusItem for Vec<u8> {
	fn from_dbus_item(item: &MessageItem) -> Option<Self> {
		match item {
			&MessageItem::Array(ref array, ref t) if t == "y" => {
				let res:Vec<Option<u8>> = array.iter()
					.map(u8::from_dbus_item).collect();

				if res.iter().all(Option::is_some) {
					Some(res.into_iter().map(Option::unwrap).collect())
				} else {
					None
				}
			},
			_ => None,
		}
	}

	fn to_dbus_item(&self) -> MessageItem {
		let vec = self.into_iter().map(u8::to_dbus_item).collect();
		MessageItem::Array(vec, Cow::Borrowed("y"))
	}
}

impl ConvertDbusItem for u8 {
	fn from_dbus_item(item: &MessageItem) -> Option<Self> {
		match item {
			&MessageItem::Byte(b) => Some(b),
			_ => None
		}
	}

	fn to_dbus_item(&self) -> MessageItem {
		MessageItem::Byte(*self)
	}
}
