// Copyright (c) 2021 Anatoly Ikorsky
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bitvec::prelude::*;

use std::io::{self};

use crate::{
    binlog::{
        BinlogCtx, BinlogEvent, BinlogStruct,
        consts::{BinlogVersion, EventType, RowsEventFlags},
    },
    io::ParseBuf,
    proto::{MyDeserialize, MySerialize},
};

use super::{RowsEvent, RowsEventRows, TableMapEvent, rows_event::RowsEventCtx};

/// Write rows event.
///
/// Used for row-based binary logging. Contains the row data to insert.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct WriteRowsEvent<'a>(RowsEvent<'a>);

impl<'a> WriteRowsEvent<'a> {
    /// Returns the number that identifies the table (see `TableMapEvent`).
    pub fn table_id(&self) -> u64 {
        self.0.table_id()
    }

    /// Returns the number of columns in the table.
    pub fn num_columns(&self) -> u64 {
        self.0.num_columns()
    }

    /// Returns columns in the after-image (only for WRITE and UPDATE).
    ///
    /// Each bit indicates whether corresponding column is used in the image.
    pub fn columns_after_image(&'a self) -> &'a BitSlice<u8> {
        self.0.columns_after_image().expect("must be here")
    }

    /// Returns raw rows data.
    pub fn rows_data(&'a self) -> &'a [u8] {
        self.0.rows_data()
    }

    /// Returns event flags (unknown bits are truncated).
    pub fn flags(&'a self) -> RowsEventFlags {
        self.0.flags()
    }

    /// Returns raw event flags (unknown bits are preserved).
    pub fn flags_raw(&'a self) -> u16 {
        self.0.flags_raw()
    }

    /// Returns an iterator over event's rows given the corresponding `TableMapEvent`.
    pub fn rows(&'a self, table_map_event: &'a TableMapEvent<'a>) -> RowsEventRows<'a> {
        RowsEventRows::new(&self.0, table_map_event, ParseBuf(self.rows_data()))
    }

    pub fn into_owned(self) -> WriteRowsEvent<'static> {
        WriteRowsEvent(self.0.into_owned())
    }
}

impl<'de> MyDeserialize<'de> for WriteRowsEvent<'de> {
    const SIZE: Option<usize> = RowsEvent::SIZE;
    type Ctx = BinlogCtx<'de>;

    fn deserialize(ctx: Self::Ctx, buf: &mut ParseBuf<'de>) -> io::Result<Self> {
        let ctx = RowsEventCtx {
            event_type: Self::EVENT_TYPE,
            binlog_ctx: ctx,
        };
        buf.parse(ctx).map(Self)
    }
}

impl MySerialize for WriteRowsEvent<'_> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.0.serialize(&mut *buf);
    }
}

impl<'a> BinlogStruct<'a> for WriteRowsEvent<'a> {
    fn len(&self, version: BinlogVersion) -> usize {
        self.0.len(version)
    }
}

impl<'a> BinlogEvent<'a> for WriteRowsEvent<'a> {
    const EVENT_TYPE: EventType = EventType::WRITE_ROWS_EVENT;
}
