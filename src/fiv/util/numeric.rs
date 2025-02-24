/*
 * fiv - Fast Image Viewer
 * Copyright 2025  Simon Arlott
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use nutype::nutype;
use std::{cmp, fmt, ops};

#[macro_export]
macro_rules! nutype_const {
	($name:ident, $ty:ty, $value:expr) => {
		const $name: $ty = match <$ty>::try_new($value) {
			Ok(value) => value,
			Err(_) => panic!("Invalid value"),
		};
	};
}

macro_rules! derive_try_into {
	($custom_from:ty, $primitive_from:ty, $custom_output:ty, $primitive_to:ty) => {
		impl TryInto<$custom_output> for $custom_from {
			type Error = anyhow::Error;

			fn try_into(self) -> Result<$custom_output, Self::Error> {
				let value = <$primitive_to>::try_from(<$primitive_from>::from(self))?;

				Ok(<$custom_output>::from(value))
			}
		}
	};
}

macro_rules! derive_numeric_op {
	($custom_lhs:ty, $custom_rhs:ty, $custom_output:ty, $primitive:ty, $trait:ty, $func:ident, $op:tt) => {
		impl $trait for $custom_lhs {
			type Output = $custom_output;

			fn $func(self, rhs: $custom_rhs) -> Self::Output {
				<$custom_output>::try_from(<$primitive>::from(self) $op <$primitive>::from(rhs)).unwrap()
			}
		}
	}
}

macro_rules! derive_numeric_op_rhs {
	($custom:ty, $primitive:ty, $trait:ty, $func:ident, $op:tt) => {
		impl $trait for $primitive {
			type Output = $custom;

			fn $func(self, rhs: $custom) -> Self::Output {
				<$custom>::try_from(self $op <$primitive>::from(rhs)).unwrap()
			}
		}
	}
}

macro_rules! derive_numeric_op_lhs {
	($custom:ty, $primitive:ty, $trait:ty, $func:ident, $op:tt) => {
		impl $trait for $custom {
			type Output = $custom;

			fn $func(self, rhs: $primitive) -> Self::Output {
				<$custom>::try_from(<$primitive>::from(self) $op rhs).unwrap()
			}
		}
	}
}

macro_rules! derive_numeric_ops_primitive {
	($custom:ty, $primitive:ty) => {
		derive_numeric_op!($custom, $custom, $custom, $primitive, ops::Add<$custom>, add, +);
		derive_numeric_op_rhs!($custom, $primitive, ops::Add<$custom>, add, +);
		derive_numeric_op_lhs!($custom, $primitive, ops::Add<$primitive>, add, +);

		derive_numeric_op!($custom, $custom, $custom, $primitive, ops::Sub<$custom>, sub, -);
		derive_numeric_op_rhs!($custom, $primitive, ops::Sub<$custom>, sub, -);
		derive_numeric_op_lhs!($custom, $primitive, ops::Sub<$primitive>, sub, -);

		derive_numeric_op!($custom, $custom, $custom, $primitive, ops::Mul<$custom>, mul, *);
		derive_numeric_op_rhs!($custom, $primitive, ops::Mul<$custom>, mul, *);
		derive_numeric_op_lhs!($custom, $primitive, ops::Mul<$primitive>, mul, *);

		derive_numeric_op!($custom, $custom, $custom, $primitive, ops::Div<$custom>, div, /);
		derive_numeric_op_rhs!($custom, $primitive, ops::Div<$custom>, div, /);
		derive_numeric_op_lhs!($custom, $primitive, ops::Div<$primitive>, div, /);
	}
}

macro_rules! derive_numeric_ops_apply {
	($custom:ty, $primitive:ty, $custom_other:ty) => {
		derive_numeric_op!($custom_other, $custom, $custom, $primitive, ops::Mul<$custom>, mul, *);
		derive_numeric_op!($custom, $custom_other, $custom, $primitive, ops::Mul<$custom_other>, mul, *);
		derive_numeric_op!($custom_other, $custom, $custom, $primitive, ops::Div<$custom>, div, /);
		derive_numeric_op!($custom, $custom_other, $custom, $primitive, ops::Div<$custom_other>, div, /);
	}
}

macro_rules! derive_numeric_op_point {
	($custom:ty, $trait:ty, $func:ident, $op:tt) => {
		impl $trait for $custom {
			type Output = $custom;

			fn $func(self, rhs: $custom) -> Self::Output {
				Self::Output::new(self.x $op rhs.x, self.y $op rhs.y)
			}
		}
	}
}

macro_rules! derive_numeric_op_xy_apply_lhs {
	($custom_lhs:ty, $custom_rhs:ty, $custom_output:ty, $x:ident, $y:ident, $trait:ty, $func:ident, $op:tt) => {
		impl $trait for $custom_lhs {
			type Output = $custom_output;

			fn $func(self, rhs: $custom_rhs) -> Self::Output {
				Self::Output::new(self $op rhs.$x, self $op rhs.$y)
			}
		}
	}
}

macro_rules! derive_numeric_op_xy_apply_rhs {
	($custom_lhs:ty, $custom_rhs:ty, $custom_output:ty, $x:ident, $y:ident, $trait:ty, $func:ident, $op:tt) => {
		impl $trait for $custom_lhs {
			type Output = $custom_output;

			fn $func(self, rhs: $custom_rhs) -> Self::Output {
				Self::Output::new(self.$x $op rhs, self.$y $op rhs)
			}
		}
	}
}

macro_rules! derive_numeric_ops_point {
	($custom:ty) => {
		derive_numeric_op_point!($custom, ops::Add<$custom>, add, +);
		derive_numeric_op_point!($custom, ops::Sub<$custom>, sub, -);
	}
}

macro_rules! derive_numeric_ops_xy_apply {
	($custom:ty, $x:ident, $y:ident, $custom_other:ty) => {
		derive_numeric_op_xy_apply_lhs!($custom_other, $custom, $custom, $x, $y, ops::Mul<$custom>, mul, *);
		derive_numeric_op_xy_apply_rhs!($custom, $custom_other, $custom, $x, $y, ops::Mul<$custom_other>, mul, *);
		derive_numeric_op_xy_apply_lhs!($custom_other, $custom, $custom, $x, $y, ops::Div<$custom>, div, /);
		derive_numeric_op_xy_apply_rhs!($custom, $custom_other, $custom, $x, $y, ops::Div<$custom_other>, div, /);
	}
}

#[nutype(
	const_fn,
	derive(
		Debug, Copy, Clone, Display, From, Into, PartialEq, Eq, PartialOrd, Ord
	)
)]
pub struct Xi32(i32);
#[nutype(
	const_fn,
	derive(
		Debug, Copy, Clone, Display, From, Into, PartialEq, Eq, PartialOrd, Ord
	)
)]
pub struct Yi32(i32);

derive_numeric_ops_primitive!(Xi32, i32);
derive_numeric_ops_primitive!(Yi32, i32);

derive_try_into!(Xi32, i32, Xu32, u32);
derive_try_into!(Yi32, i32, Yu32, u32);

#[nutype(
	const_fn,
	derive(
		Debug, Copy, Clone, Display, From, Into, PartialEq, Eq, PartialOrd, Ord
	)
)]
pub struct Xu32(u32);
#[nutype(
	const_fn,
	derive(
		Debug, Copy, Clone, Display, From, Into, PartialEq, Eq, PartialOrd, Ord
	)
)]
pub struct Yu32(u32);

derive_numeric_ops_primitive!(Xu32, u32);
derive_numeric_ops_primitive!(Yu32, u32);

derive_try_into!(Xu32, u32, Xi32, i32);
derive_try_into!(Yu32, u32, Yi32, i32);

impl From<Xu32> for f64 {
	fn from(value: Xu32) -> Self {
		f64::from(u32::from(value))
	}
}

impl From<Yu32> for f64 {
	fn from(value: Yu32) -> Self {
		f64::from(u32::from(value))
	}
}

#[nutype(
	const_fn,
	validate(finite),
	derive(Debug, Copy, Clone, TryFrom, Into, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct Xf64(f64);
#[nutype(
	const_fn,
	validate(finite),
	derive(Debug, Copy, Clone, TryFrom, Into, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct Yf64(f64);

derive_numeric_ops_primitive!(Xf64, f64);
derive_numeric_ops_primitive!(Yf64, f64);

impl From<Xi32> for Xf64 {
	fn from(value: Xi32) -> Self {
		Xf64::try_from(f64::from(i32::from(value))).unwrap()
	}
}

impl From<Yi32> for Yf64 {
	fn from(value: Yi32) -> Self {
		Yf64::try_from(f64::from(i32::from(value))).unwrap()
	}
}

impl From<Xu32> for Xf64 {
	fn from(value: Xu32) -> Self {
		Xf64::try_from(f64::from(u32::from(value))).unwrap()
	}
}

impl From<Yu32> for Yf64 {
	fn from(value: Yu32) -> Self {
		Yf64::try_from(f64::from(u32::from(value))).unwrap()
	}
}

impl fmt::Display for Xf64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:.2}", self.into_inner())
	}
}

impl fmt::Display for Yf64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:.2}", self.into_inner())
	}
}

/// "Everything" trait for Xf64 and Yf64, so that either type can be used as
/// a generic type parameter
pub trait XYf64<T>:
	Copy
	+ Clone
	+ TryFrom<f64>
	+ Into<f64>
	+ Zero
	+ ops::Add<T, Output = T>
	+ ops::Add<f64, Output = T>
	+ ops::Sub<T, Output = T>
	+ ops::Sub<f64, Output = T>
	+ ops::Mul<T, Output = T>
	+ ops::Mul<f64, Output = T>
	+ ops::Mul<Sf64, Output = T>
	+ ops::Div<T, Output = T>
	+ ops::Div<f64, Output = T>
	+ ops::Div<Sf64, Output = T>
	+ cmp::PartialEq
	+ cmp::Eq
	+ cmp::PartialOrd<T>
	+ cmp::Ord
	+ fmt::Display
{
}

impl XYf64<Xf64> for Xf64 {}

impl XYf64<Yf64> for Yf64 {}

pub trait Zero {
	fn zero() -> Self;
}

impl Zero for Xf64 {
	fn zero() -> Self {
		Xf64::try_from(0.0).unwrap()
	}
}

impl Zero for Yf64 {
	fn zero() -> Self {
		Yf64::try_from(0.0).unwrap()
	}
}

#[nutype(
	const_fn,
	validate(finite),
	derive(
		Debug, Copy, Clone, Display, TryFrom, Into, PartialEq, Eq, PartialOrd, Ord
	)
)]
pub struct Sf64(f64);

derive_numeric_ops_primitive!(Sf64, f64);
derive_numeric_ops_apply!(Xf64, f64, Sf64);
derive_numeric_ops_apply!(Yf64, f64, Sf64);

impl Sf64 {
	pub fn actual() -> Self {
		Sf64::try_from(1.0).unwrap()
	}

	pub fn ratio<T: XYf64<T>>(num: T, denom: T) -> Self {
		Sf64::try_from((num / denom).into()).unwrap()
	}
}

#[derive(Debug, Copy, Clone, PartialEq, derive_more::Constructor)]
pub struct PointI32 {
	pub x: Xi32,
	pub y: Yi32,
}

derive_numeric_ops_point!(PointI32);

impl Default for PointI32 {
	fn default() -> Self {
		Self {
			x: 0.into(),
			y: 0.into(),
		}
	}
}

impl From<(i32, i32)> for PointI32 {
	fn from(value: (i32, i32)) -> Self {
		Self::new(value.0.into(), value.1.into())
	}
}

impl fmt::Display for PointI32 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:.2},{:.2}", self.x.into_inner(), self.y.into_inner())
	}
}

#[derive(Debug, Copy, Clone, PartialEq, derive_more::Constructor)]
pub struct PointF64 {
	pub x: Xf64,
	pub y: Yf64,
}

derive_numeric_ops_point!(PointF64);
derive_numeric_ops_xy_apply!(PointF64, x, y, Sf64);

impl Default for PointF64 {
	fn default() -> Self {
		Self {
			x: Xf64::zero(),
			y: Yf64::zero(),
		}
	}
}

impl From<(f64, f64)> for PointF64 {
	fn from(value: (f64, f64)) -> Self {
		Self::new(
			Xf64::try_from(value.0).unwrap(),
			Yf64::try_from(value.1).unwrap(),
		)
	}
}

impl From<PointI32> for PointF64 {
	fn from(value: PointI32) -> Self {
		Self::new(value.x.into(), value.y.into())
	}
}

impl fmt::Display for PointF64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:.2},{:.2}", self.x.into_inner(), self.y.into_inner())
	}
}

#[derive(Debug, Copy, Clone, PartialEq, derive_more::Constructor)]
pub struct DimensionsU32 {
	pub width: Xu32,
	pub height: Yu32,
}

impl DimensionsU32 {
	pub fn rotate90(self) -> Self {
		Self::new(u32::from(self.height).into(), u32::from(self.width).into())
	}
}

impl From<(u32, u32)> for DimensionsU32 {
	fn from(value: (u32, u32)) -> Self {
		Self::new(Xu32::new(value.0), Yu32::new(value.1))
	}
}

impl From<&gtk::Allocation> for DimensionsU32 {
	fn from(allocation: &gtk::Allocation) -> Self {
		Self::new(
			Xu32::new(u32::try_from(allocation.width()).unwrap()),
			Yu32::new(u32::try_from(allocation.height()).unwrap()),
		)
	}
}

impl fmt::Display for DimensionsU32 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{:.2}x{:.2}",
			self.width.into_inner(),
			self.height.into_inner()
		)
	}
}

#[derive(Debug, Copy, Clone, PartialEq, derive_more::Constructor)]
pub struct DimensionsF64 {
	pub width: Xf64,
	pub height: Yf64,
}

derive_numeric_ops_xy_apply!(DimensionsF64, width, height, Sf64);

impl DimensionsF64 {
	pub fn centre(&self) -> PointF64 {
		PointF64::new(self.width / 2.0, self.height / 2.0)
	}
}

impl From<DimensionsU32> for DimensionsF64 {
	fn from(value: DimensionsU32) -> Self {
		Self::new(value.width.into(), value.height.into())
	}
}

impl From<&gtk::Allocation> for DimensionsF64 {
	fn from(allocation: &gtk::Allocation) -> Self {
		DimensionsU32::from(allocation).into()
	}
}

impl fmt::Display for DimensionsF64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{:.2}x{:.2}",
			self.width.into_inner(),
			self.height.into_inner()
		)
	}
}

impl cmp::PartialOrd for DimensionsF64 {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		if self == other {
			Some(cmp::Ordering::Equal)
		} else if self.width < other.width && self.height < other.height {
			Some(cmp::Ordering::Less)
		} else if self.width > other.width && self.height > other.height {
			Some(cmp::Ordering::Greater)
		} else {
			None
		}
	}
}
