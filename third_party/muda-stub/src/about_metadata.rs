//! Metadata for "About" predefined menu items.

/// Metadata for the `about` predefined menu item (mirrors
/// `muda::AboutMetadata`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AboutMetadata {
    /// Application name.
    pub name: Option<String>,
    /// Application version.
    pub version: Option<String>,
    /// Short version (macOS).
    pub short_version: Option<String>,
    /// Authors.
    pub authors: Option<Vec<String>>,
    /// Comments.
    pub comments: Option<String>,
    /// Copyright notice.
    pub copyright: Option<String>,
    /// License name.
    pub license: Option<String>,
    /// Website URL.
    pub website: Option<String>,
    /// Website label.
    pub website_label: Option<String>,
    /// Credits.
    pub credits: Option<String>,
    /// Application icon (retained, never rendered).
    pub icon: Option<crate::icon::Icon>,
}

impl AboutMetadata {
    /// Returns a builder over the metadata.
    pub fn builder() -> AboutMetadataBuilder {
        AboutMetadataBuilder::new()
    }
}

/// Builder for [`AboutMetadata`].
#[derive(Debug, Default)]
pub struct AboutMetadataBuilder(AboutMetadata);

impl AboutMetadataBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self(AboutMetadata::default())
    }

    /// Sets the application name.
    pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        self.0.name = name.map(Into::into);
        self
    }

    /// Sets the application version.
    pub fn version<S: Into<String>>(mut self, version: Option<S>) -> Self {
        self.0.version = version.map(Into::into);
        self
    }

    /// Sets the short version.
    pub fn short_version<S: Into<String>>(mut self, short_version: Option<S>) -> Self {
        self.0.short_version = short_version.map(Into::into);
        self
    }

    /// Sets the authors.
    pub fn authors(mut self, authors: Option<Vec<String>>) -> Self {
        self.0.authors = authors;
        self
    }

    /// Sets the comments.
    pub fn comments<S: Into<String>>(mut self, comments: Option<S>) -> Self {
        self.0.comments = comments.map(Into::into);
        self
    }

    /// Sets the copyright.
    pub fn copyright<S: Into<String>>(mut self, copyright: Option<S>) -> Self {
        self.0.copyright = copyright.map(Into::into);
        self
    }

    /// Sets the license.
    pub fn license<S: Into<String>>(mut self, license: Option<S>) -> Self {
        self.0.license = license.map(Into::into);
        self
    }

    /// Sets the website URL.
    pub fn website<S: Into<String>>(mut self, website: Option<S>) -> Self {
        self.0.website = website.map(Into::into);
        self
    }

    /// Sets the website label.
    pub fn website_label<S: Into<String>>(mut self, website_label: Option<S>) -> Self {
        self.0.website_label = website_label.map(Into::into);
        self
    }

    /// Sets the credits.
    pub fn credits<S: Into<String>>(mut self, credits: Option<S>) -> Self {
        self.0.credits = credits.map(Into::into);
        self
    }

    /// Sets the icon.
    pub fn icon(mut self, icon: Option<crate::icon::Icon>) -> Self {
        self.0.icon = icon;
        self
    }

    /// Builds the metadata.
    pub fn build(self) -> AboutMetadata {
        self.0
    }
}
