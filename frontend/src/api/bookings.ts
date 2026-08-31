import type { Booking } from "../bindings/Booking";
import type { CreateBookingParams } from "../bindings/CreateBookingParams";
import type { Page } from "../bindings/Page";
import { del, get, post } from "./client";

export function listBookings(page = 1, pageSize = 20): Promise<Page<Booking>> {
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  });
  return get<Page<Booking>>(`/api/bookings?${params.toString()}`);
}

export function createBooking(body: CreateBookingParams): Promise<Booking> {
  return post<Booking>("/api/bookings", body);
}

export function cancelBooking(id: number): Promise<void> {
  return del(`/api/bookings/${id}`);
}
