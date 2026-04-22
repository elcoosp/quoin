//! Framework‑specific reactive context.
//!
//! This module defines the [`ReactiveContext`] trait, which is the primary
//! entry point for creating signals and accessing the async executor within
//! a UI framework. Adapter crates implement this trait for their specific
//! runtime.
//!
//! # Global State (`provide_global` / `use_global`)
//!
//! The global state methods allow dependency injection of shared values
//! across component trees. Because each framework has a different context
//! propagation model, the behavior differs by adapter:
//!
//! | Framework  | Storage mechanism                                | Lifetime                            |
//! |------------|---------------------------------------------------|--------------------------------------|
//! | **GPUI**   | Thread‑local `HashMap<TypeId, Box<dyn Any>>`      | Thread‑scoped; cleared on thread exit |
//! | **Leptos** | `provide_context` / `use_context` on the reactive owner tree | Scoped to the nearest `Owner` / component tree |
//! | **Dioxus**  | `provide_context` / `try_consume_context` on the Dioxus scope | Scoped to the component's `ScopeId` |
//!
//! **Important:** In GPUI, globals are stored in a *thread-local* map. This
//! means they are only accessible from the same thread that called
//! `provide_global`. If your application spawns views on multiple threads,
//! you must call `provide_global` on each thread separately. In Leptos and
//! Dioxus, context follows the reactive/component tree, so a single
//! `provide_global` at the root is sufficient for all descendants.

use crate::{Executor, Signal};

/// A framework-specific reactive runtime context.
///
/// `ReactiveContext` abstracts over the reactive primitives of a UI framework.
/// It provides methods to create signals, obtain the async executor, and
/// request UI updates.
///
/// # Implementing `ReactiveContext`
///
/// Adapter crates must implement this trait for their context type. The
/// associated types `Signal` and `Executor` should be the framework's native
/// signal and executor types, respectively.
///
/// # Example
///
/// ```rust,ignore
/// use quoin_core::ReactiveContext;
///
/// fn create_counter<C: ReactiveContext>(cx: &C) -> C::Signal<u32> {
///     cx.create_signal(0u32)
/// }
/// ```
pub trait ReactiveContext: Clone + Send + Sync + 'static {
    /// The framework's native signal type.
    ///
    /// This type must implement the [`Signal`] trait and be `Clone`.
    type Signal<T: Clone + 'static>: Signal<T>;

    /// The framework's async executor.
    ///
    /// This type must implement the [`Executor`] trait.
    type Executor: Executor;

    /// Creates a new signal with the given initial value.
    ///
    /// The signal is owned by the current reactive scope. It will be
    /// automatically cleaned up when the scope is disposed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = cx.create_signal(0u32);
    /// assert_eq!(count.get(), 0);
    /// ```
    fn create_signal<T: Clone + 'static>(&self, initial: T) -> Self::Signal<T>;

    /// Returns the executor for spawning asynchronous tasks.
    ///
    /// Use this executor to run background work without blocking the UI.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let executor = cx.executor();
    /// executor.spawn(async {
    ///     // Long‑running operation
    /// }).detach();
    /// ```
    fn executor(&self) -> Self::Executor;

    /// Requests that the UI re‑render.
    ///
    /// Some frameworks require an explicit call to mark the UI as dirty.
    /// This method provides a hook for that purpose. In frameworks with
    /// automatic reactivity, this may be a no‑op.
    fn request_update(&self);

    /// Provide a global value that can be retrieved via [`use_global`](Self::use_global).
    ///
    /// The value is stored in a framework-specific context store. Subsequent
    /// calls to [`use_global`](Self::use_global) with the same type will
    /// return a signal wrapping this value.
    ///
    /// # Framework-Specific Behavior
    ///
    /// - **GPUI**: Stores the value in a **thread-local** `HashMap<TypeId, Box<dyn Any>>`.
    ///   The global is only visible to code running on the **same thread** that called
    ///   `provide_global`. If your application opens windows on multiple threads, you
    ///   must call `provide_global` on each thread. There is no automatic cleanup when
    ///   a window closes — consider calling a manual cleanup in `on_unmount` if needed.
    ///
    /// - **Leptos**: Wraps the value in `RwSignal<SendWrapper<T>>` and calls
    ///   `provide_context`. The global is scoped to the current reactive [`Owner`],
    ///   so it is automatically cleaned up when the owner is dropped (e.g., when a
    ///   component unmounts). A single `provide_global` at the app root propagates
    ///   to all descendant components.
    ///
    /// - **Dioxus**: Calls `provide_context` on the current Dioxus scope. The global
    ///   is scoped to the component's [`ScopeId`] and its descendants. It is cleaned
    ///   up when the scope is released.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // At app root:
    /// cx.provide_global(Theme::Dark);
    ///
    /// // In any descendant:
    /// if let Some(theme) = cx.use_global::<Theme>() {
    ///     let current = theme.get();
    /// }
    /// ```
    fn provide_global<T: Clone + Send + Sync + 'static>(&self, value: T);

    /// Retrieves a global signal from the framework's context provider.
    ///
    /// Returns `Some(signal)` if a value of type `T` has been provided
    /// via [`provide_global`](Self::provide_global), `None` otherwise.
    ///
    /// # Framework-Specific Behavior
    ///
    /// - **GPUI**: Looks up the value in the **thread-local** type-map. Returns
    ///   `None` if no value of type `T` has been registered on the **current thread**,
    ///   even if it was registered on a different thread. The returned signal wraps
    ///   the value in a new `Arc<RwLock<T>>` — mutations to this signal do **not**
    ///   propagate back to the stored global (each `use_global` call creates an
    ///   independent copy). If you need shared mutability, consider providing a
    ///   signal type directly.
    ///
    /// - **Leptos**: Calls `use_context::<RwSignal<SendWrapper<T>>>()`. Returns
    ///   `None` if no matching context exists in the current reactive owner tree.
    ///   The returned `LeptosSignal` shares the same underlying `RwSignal`, so
    ///   mutations are visible to all holders of the signal.
    ///
    /// - **Dioxus**: Calls `try_consume_context::<T>()`. Returns `None` if no
    ///   matching context exists in the current scope. Note that Dioxus contexts
    ///   may be consumed (removed) by the first `use_context` call depending on
    ///   the Dioxus version; `try_consume_context` is used to avoid this.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(theme) = cx.use_global::<Theme>() {
    ///     let current = theme.get();
    /// }
    /// ```
    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>>;
}
