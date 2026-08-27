#![allow(unused_macros)]
/// Forwards an owned assignment operation to its reference implementation.
///
/// For example:
///
/// ```ignore
/// forward_val_assign!(impl AddAssign for BoundedRational, add_assign);
/// ```
///
/// generates the `BoundedRational += BoundedRational` implementation
/// from the already implemented `BoundedRational += &BoundedRational`.
macro_rules! forward_val_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            #[inline]
            fn $method(&mut self, other: $res) {
                self.$method(&other);
            }
        }
    };
}

/// Generates `Add` implementations for smaller scalar types by promoting
/// them to a larger scalar type.
///
/// The promoted implementation must already exist.
///
/// For example:
///
/// ```ignore
/// promote_scalars!(impl Add<u32> for BoundedRational, add, u8, u16);
/// ```
///
/// generates `BoundedRational + u8` and `BoundedRational + u16` by
/// converting those values to `u32`.
macro_rules! promote_scalars {
    (
        impl $imp:ident<$promo:ty> for $res:ty,
        $method:ident,
        $($scalar:ty),*
    ) => {
        $(
            forward_all_scalar_binop_to_val_val!(
                impl $imp<$scalar> for $res,
                $method
            );

            impl $imp<$scalar> for $res {
                type Output = $res;

                #[inline]
                fn $method(self, other: $scalar) -> $res {
                    $imp::$method(self, other as $promo)
                }
            }
        )*
    };
}

/// Generates `AddAssign` implementations for smaller scalar types by
/// promoting them to a larger scalar type.
///
/// The promoted implementation must already exist.
macro_rules! promote_scalars_assign {
    (
        impl $imp:ident<$promo:ty> for $res:ty,
        $method:ident,
        $($scalar:ty),*
    ) => {
        $(
            impl $imp<$scalar> for $res {
                #[inline]
                fn $method(&mut self, other: $scalar) {
                    self.$method(other as $promo);
                }
            }
        )*
    };
}

/// Promotes unsigned scalar types.
///
/// `u8` and `u16` are promoted to `u32`.
/// `usize` is promoted to `u64`.
macro_rules! promote_unsigned_scalars {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars!(
            impl $imp<u32> for $res,
            $method,
            u8,
            u16
        );

        promote_scalars!(
            impl $imp<u64> for $res,
            $method,
            usize
        );
    };
}

/// Promotes unsigned scalar types for assignment operations.
macro_rules! promote_unsigned_scalars_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars_assign!(
            impl $imp<u32> for $res,
            $method,
            u8,
            u16
        );

        promote_scalars_assign!(
            impl $imp<u64> for $res,
            $method,
            usize
        );
    };
}

/// Promotes signed scalar types.
///
/// `i8` and `i16` are promoted to `i32`.
/// `isize` is promoted to `i64`.
macro_rules! promote_signed_scalars {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars!(
            impl $imp<i32> for $res,
            $method,
            i8,
            i16
        );

        promote_scalars!(
            impl $imp<i64> for $res,
            $method,
            isize
        );
    };
}

/// Promotes signed scalar types for assignment operations.
macro_rules! promote_signed_scalars_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars_assign!(
            impl $imp<i32> for $res,
            $method,
            i8,
            i16
        );

        promote_scalars_assign!(
            impl $imp<i64> for $res,
            $method,
            isize
        );
    };
}

/// Promotes all unsigned and signed scalar types.
macro_rules! promote_all_scalars {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_unsigned_scalars!(
            impl $imp for $res,
            $method
        );

        promote_signed_scalars!(
            impl $imp for $res,
            $method
        );
    };
}

/// Promotes all unsigned and signed scalar types for assignment operations.
macro_rules! promote_all_scalars_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_unsigned_scalars_assign!(
            impl $imp for $res,
            $method
        );

        promote_signed_scalars_assign!(
            impl $imp for $res,
            $method
        );
    };
}

/// Forwards a scalar/value operation to the corresponding value/value
/// implementation.
macro_rules! forward_scalar_val_val_binop_commutative {
    (
        impl $imp:ident<$scalar:ty> for $res:ty,
        $method:ident
    ) => {
        impl $imp<$res> for $scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}

/// Generates all reference combinations of `res op scalar` from the
/// already implemented `res op scalar` (value/value) operation.
///
/// This includes:
///
/// - `&BoundedRational + scalar`
/// - `BoundedRational + &scalar`
/// - `&BoundedRational + &scalar`
///
/// Unlike `forward_all_scalar_binop_to_val_val_commutative!`, this does
/// NOT generate `scalar + BoundedRational` implementations.
macro_rules! forward_all_scalar_binop_to_val_val {
    (
        impl $imp:ident<$scalar:ty> for $res:ty,
        $method:ident
    ) => {
        impl $imp<$scalar> for &$res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $scalar) -> $res {
                $imp::$method(self.clone(), other)
            }
        }

        impl $imp<&$scalar> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(self, *other)
            }
        }

        impl $imp<&$scalar> for &$res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(self.clone(), *other)
            }
        }
    };
}

/// Generates all scalar/value combinations for a commutative operation.
///
/// This includes:
///
/// - `scalar + BoundedRational`
/// - `scalar + &BoundedRational`
/// - `&scalar + BoundedRational`
/// - `&scalar + &BoundedRational`
macro_rules! forward_all_scalar_binop_to_val_val_commutative {
    (
        impl $imp:ident<$scalar:ty> for $res:ty,
        $method:ident
    ) => {
        forward_scalar_val_val_binop_commutative!(
            impl $imp<$scalar> for $res,
            $method
        );

        impl $imp<&$res> for $scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other.clone(), self)
            }
        }

        impl $imp<$res> for &$scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, *self)
            }
        }

        impl $imp<&$res> for &$scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other.clone(), *self)
            }
        }
    };
}

/// Implements `Sum` for a result type for every input type that can be
/// added to it.
///
/// The result type must provide a zero value.
///
/// This is equivalent in purpose to `num-bigint`'s `impl_sum_iter_type!`,
/// which starts the fold from its zero value. :contentReference[oaicite:2]{index=2}
macro_rules! impl_sum_iter_type {
    ($res:ty) => {
        impl<T> Sum<T> for $res
        where
            $res: Add<T, Output = $res>,
        {
            fn sum<I>(iter: I) -> Self
            where
                I: Iterator<Item = T>,
            {
                iter.fold(<$res>::from_bigint(ZERO.clone()), <$res as Add<T>>::add)
            }
        }
    };
}
