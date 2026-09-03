#![allow(unused_macros)]

/// Forwards an owned-value assignment to the reference-based implementation.
///
/// Generates:
///     T op= T
/// by forwarding to:
///     T op= &T
///
/// The existing `&T` implementation contains the actual logic, so the
/// owned-value implementation only needs to borrow `other`.
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

/// Generates scalar implementations using a promoted scalar type.
///
/// For every `$scalar`, generates:
///     T op scalar
///     scalar op T
/// and forwards reference combinations through
/// `forward_all_scalar_binop_to_val_val!`.
///
/// The scalar is converted:
///     scalar -> promo
///
/// The actual operation is then delegated to an implementation involving
/// the promoted scalar type.
macro_rules! promote_scalars {
    (impl $imp:ident<$promo:ty> for $res:ty, $method:ident, $( $scalar:ty ),*) => {
        $(
            forward_all_scalar_binop_to_val_val!(impl $imp<$scalar> for $res, $method);

            impl $imp<$scalar> for $res {
                type Output = $res;

                #[allow(clippy::cast_lossless)]
                #[inline]
                fn $method(self, other: $scalar) -> $res {
                    $imp::$method(self, other as $promo)
                }
            }

            impl $imp<$res> for $scalar {
                type Output = $res;

                #[allow(clippy::cast_lossless)]
                #[inline]
                fn $method(self, other: $res) -> $res {
                    $imp::$method(self as $promo, other)
                }
            }
        )*
    }
}

/// Generates scalar assignment implementations.
///
/// For every `$scalar`, generates:
///     T op= scalar
/// And for every `$scalar`, generates:
///     T op= &scalar
/// The scalar is converted:
///     scalar -> promo
/// and forwarded to the existing assignment implementation.
macro_rules! promote_scalars_assign {
    (impl $imp:ident<$promo:ty> for $res:ty, $method:ident, $( $scalar:ty ),*) => {
        $(
            impl $imp<$scalar> for $res {
                #[allow(clippy::cast_lossless)]
                #[inline]
                fn $method(&mut self, other: $scalar) {
                    self.$method(other as $promo);
                }
            }

            impl $imp<&$scalar> for $res {
                #[allow(clippy::cast_lossless)]
                #[inline]
                fn $method(&mut self, other: &$scalar) {
                    self.$method(*other as $promo);
                }
            }
        )*
    }
}

/// Expands `promote_scalars!` for unsigned scalar types.
///
/// Generates support for:
///     T op u8
///     T op u16
///     T op usize
/// and the corresponding reverse/reference combinations.
macro_rules! promote_unsigned_scalars {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars!(impl $imp<u32> for $res, $method, u8, u16);
        promote_scalars!(impl $imp<UsizePromotion> for $res, $method, usize);
    }
}

/// Expands `promote_scalars_assign!` for unsigned scalar types.
///
/// Generates:
///     T op= u8
///     T op= u16
///     T op= usize
macro_rules! promote_unsigned_scalars_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars_assign!(impl $imp<u32> for $res, $method, u8, u16);
        promote_scalars_assign!(impl $imp<UsizePromotion> for $res, $method, usize);
    }
}

/// Expands `promote_scalars!` for signed scalar types.
///
/// Generates support for:
///     T op i8
///     T op i16
///     T op isize
/// and the corresponding reverse/reference combinations.
macro_rules! promote_signed_scalars {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars!(impl $imp<i32> for $res, $method, i8, i16);
        promote_scalars!(impl $imp<IsizePromotion> for $res, $method, isize);
    }
}

/// Expands `promote_scalars_assign!` for signed scalar types.
///
/// Generates:
///     T op= i8
///     T op= i16
///     T op= isize
macro_rules! promote_signed_scalars_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_scalars_assign!(impl $imp<i32> for $res, $method, i8, i16);
        promote_scalars_assign!(impl $imp<IsizePromotion> for $res, $method, isize);
    }
}

/// Combines signed and unsigned scalar implementations.
///
/// Expands:
///     promote_unsigned_scalars!
///     promote_signed_scalars!
macro_rules! promote_all_scalars {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_unsigned_scalars!(impl $imp for $res, $method);
        promote_signed_scalars!(impl $imp for $res, $method);
    }
}

/// Combines signed and unsigned scalar assignment implementations.
///
/// Expands:
///     promote_unsigned_scalars_assign!
///     promote_signed_scalars_assign!
macro_rules! promote_all_scalars_assign {
    (impl $imp:ident for $res:ty, $method:ident) => {
        promote_unsigned_scalars_assign!(impl $imp for $res, $method);
        promote_signed_scalars_assign!(impl $imp for $res, $method);
    }
}

/// Generates the reverse owned-value combination for a commutative operation.
///
/// Given:
///     T op scalar
/// generates:
///     scalar op T
/// by forwarding to:
///     T op scalar
///
/// Use only when reversing the operands preserves the result.
macro_rules! forward_scalar_val_val_binop_commutative {
    (impl $imp:ident < $scalar:ty > for $res:ty, $method:ident) => {
        impl $imp<$res> for $scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, self)
            }
        }
    };
}

/// Forwards combinations containing a scalar reference.
///
/// Generates:
///     T op &scalar
///     &scalar  op T
/// by forwarding to:
///     T op scalar
///     scalar op T
macro_rules! forward_scalar_val_ref_binop_to_val_val {
    (impl $imp:ident<$scalar:ty> for $res:ty, $method:ident) => {
        impl $imp<&$scalar> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(self, *other)
            }
        }

        impl $imp<$res> for &$scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(*self, other)
            }
        }
    };
}

/// Forwards combinations containing a reference to the result type.
///
/// Generates:
///     &T op scalar
///     scalar op &T
/// by converting `&T` to `T` using `clone()`.
macro_rules! forward_scalar_ref_val_binop_to_val_val {
    (impl $imp:ident < $scalar:ty > for $res:ty, $method:ident) => {
        impl $imp<$scalar> for &$res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $scalar) -> $res {
                $imp::$method(self.clone(), other)
            }
        }

        impl $imp<&$res> for $scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(self, other.clone())
            }
        }
    };
}

/// Forwards combinations where both operands are references.
///
/// Generates:
///     &T op &scalar
///     &scalar op &T
/// by converting:
///     &T      -> T
///     &scalar -> scalar
/// and forwarding to the corresponding owned-value implementation.
macro_rules! forward_scalar_ref_ref_binop_to_val_val {
    (impl $imp:ident<$scalar:ty> for $res:ty, $method:ident) => {
        impl $imp<&$scalar> for &$res {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(self.clone(), *other)
            }
        }

        impl $imp<&$res> for &$scalar {
            type Output = $res;

            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(*self, other.clone())
            }
        }
    };
}

/// Combines all scalar/reference forwarding combinations.
///
/// Expands:
///     T op &scalar
///     &scalar op T
///     &T op scalar
///     scalar op &T
///     &T op &scalar
///     &scalar op &T
///
/// All combinations are forwarded to the corresponding owned-value
/// implementations.
macro_rules! forward_all_scalar_binop_to_val_val {
    (impl $imp:ident<$scalar:ty> for $res:ty, $method:ident) => {
        forward_scalar_val_ref_binop_to_val_val!(impl $imp<$scalar> for $res, $method);
        forward_scalar_ref_val_binop_to_val_val!(impl $imp<$scalar> for $res, $method);
        forward_scalar_ref_ref_binop_to_val_val!(impl $imp<$scalar> for $res, $method);
    }
}

/// Combines commutative forwarding with scalar/reference forwarding.
///
/// Generates:
///     scalar op T
///     T op &scalar
///     &scalar op T
///     &T op scalar
///     scalar op &T
///     &T op &scalar
///     &scalar op &T
///
/// The reverse owned-value combination:
///     scalar op T
/// is generated using `forward_scalar_val_val_binop_commutative!`.
macro_rules! forward_all_scalar_binop_to_val_val_commutative {
    (impl $imp:ident<$scalar:ty> for $res:ty, $method:ident) => {
        forward_scalar_val_val_binop_commutative!(impl $imp<$scalar> for $res, $method);
        forward_all_scalar_binop_to_val_val!(impl $imp<$scalar> for $res, $method);
    }
}

/// Implements `Sum` for `$res`.
/// The iterator item type is `T`.
///
/// Requires:
///     $res: Add<T, Output = $res>
/// Starts with:
///     $res::from_bigint(ZERO.clone())
/// and folds:
///     zero
///     zero op item1
///     (zero op item1) op item2
///     ...
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
