use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dtos::classes::Class;
use crate::models::_entities::{bookings, classes};

/// A booking as returned to the person who made it. The booked `class` is
/// embedded so "my bookings" renders without a second round-trip; `class_id`
/// is kept flat for convenience.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct Booking {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub class_id: i64,
    /// `booked` (holds a seat) or `waitlisted` (in line for one).
    pub status: String,
    /// `pending` until the (mock) payment succeeds, then `paid`.
    pub payment_status: String,
    pub created_at: String,
    pub class: Class,
}

impl Booking {
    #[must_use]
    pub fn from_parts(booking: bookings::Model, class: classes::Model, class_booked: i64) -> Self {
        Self {
            id: booking.id,
            class_id: booking.class_id,
            status: booking.status,
            payment_status: booking.payment_status,
            created_at: booking.created_at.to_rfc3339(),
            class: Class::from_parts(class, class_booked),
        }
    }
}

/// Body for making a booking. The booker is the authenticated user (from the
/// JWT), so only the target class is supplied.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../frontend/src/bindings/")]
pub struct CreateBookingParams {
    #[ts(type = "number")]
    pub class_id: i64,
}
