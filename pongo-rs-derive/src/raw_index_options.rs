use darling::FromMeta;

#[derive(Clone, Debug)]
pub(crate) struct CustomDocument(mongodb::bson::Document);

impl FromMeta for CustomDocument {
    fn from_string(value: &str) -> darling::Result<Self> {
        println!("{value:#?}");
        let value = mongodb::bson::from_slice(value.as_bytes());
        match value {
            Ok(document) => Ok(Self(document)),
            Err(error) => Err(darling::Error::unsupported_shape(&format!("{error}"))),
        }
    }
}

#[derive(Debug, Default, FromMeta)]
/// Mapped IndexOptions
pub(crate) struct RawIndexOptions {
    /// Tells the server to build the index in the background and not block other tasks. Starting
    /// in MongoDB 4.2, this option is deprecated and ignored by the server.
    #[darling(default)]
    pub(crate) background: Option<bool>,
    /// Specifies a TTL to control how long MongoDB retains
    /// documents in this collection.
    ///
    /// See the [documentation](https://docs.mongodb.com/manual/core/index-ttl/)
    /// for more information on how to use this option.
    #[darling(default)]
    pub(crate) expire_after: Option<u64>,

    /// Specifies a name outside the default generated name.
    #[darling(default)]
    pub(crate) name: Option<String>,

    /// If true, the index only references documents with the specified field. The
    /// default value is false.
    ///
    /// See the [documentation](https://docs.mongodb.com/manual/core/index-sparse/)
    /// for more information on how to use this option.
    #[darling(default)]
    pub(crate) sparse: Option<bool>,

    /// Allows users to configure the storage engine on a per-index basis when creating
    /// an index.
    #[darling(default)]
    pub(crate) storage_engine: Option<CustomDocument>,
    /// Forces the index to be unique so the collection will not accept documents where the index
    /// key value matches an existing value in the index. The default value is false.
    #[darling(default)]
    pub(crate) unique: Option<bool>,

    /// Specify the version number of the index.
    /// Starting in MongoDB 3.2, Version 0 indexes are not allowed.
    #[darling(default)]
    pub(crate) version: Option<u32>,

    /// For text indexes, the language that determines the list of stop words and the
    /// rules for the stemmer and tokenizer.
    #[darling(default)]
    pub(crate) default_language: Option<String>,

    /// For `text` indexes, the name of the field, in the collection’s documents, that
    /// contains the override language for the document.
    #[darling(default)]
    pub(crate) language_override: Option<String>,

    /// The `text` index version number. Users can use this option to override the default
    /// version number.
    #[darling(default)]
    pub(crate) text_index_version: Option<u32>,

    /// For `text` indexes, a document that contains field and weight pairs.
    #[darling(default)]
    pub(crate) weights: Option<CustomDocument>,

    /// The `2dsphere` index version number.
    /// As of MongoDB 3.2, version 3 is the default. Version 2 is the default in MongoDB 2.6 and
    /// 3.0 series.
    #[darling(default)]
    pub(crate) sphere_2d_index_version: Option<u32>,

    /// For `2dsphere` indexes, the number of precision of the stored geohash value of the
    /// location data. The bits value ranges from 1 to 32 inclusive.
    #[darling(default)]
    pub(crate) bits: Option<u32>,

    /// For `2dsphere` indexes, the upper inclusive boundary for the longitude and latitude
    /// values.
    #[darling(default)]
    pub(crate) max: Option<f64>,

    /// For `2dsphere` indexes, the lower inclusive boundary for the longitude and latitude
    /// values.
    #[darling(default)]
    pub(crate) min: Option<f64>,

    /// For `geoHaystack` indexes, specify the number of units within which to group the location
    /// values.
    #[darling(default)]
    pub(crate) bucket_size: Option<u32>,

    /// If specified, the index only references documents that match the filter
    /// expression. See Partial Indexes for more information.
    #[darling(default)]
    pub(crate) partial_filter_expression: Option<CustomDocument>,

    // /// Specifies the collation for the index.
    // #[darling(default)]
    // pub collation: Option<mongodb::options::Collation>,
    /// Allows users to include or exclude specific field paths from a wildcard index.
    #[darling(default)]
    pub(crate) wildcard_projection: Option<CustomDocument>,

    /// A flag that determines whether the index is hidden from the query planner. A
    /// hidden index is not evaluated as part of the query plan selection.
    #[darling(default)]
    pub(crate) hidden: Option<bool>,
}

impl Into<mongodb::options::IndexOptions> for &RawIndexOptions {
    fn into(self) -> mongodb::options::IndexOptions {
        let builder = mongodb::options::IndexOptions::builder();

        builder
            .background(self.background)
            .expire_after(
                self.expire_after
                    .map_or(None, |expire| Some(std::time::Duration::from_secs(expire))),
            )
            .name(self.name.clone())
            .sparse(self.sparse)
            .storage_engine(
                self.storage_engine
                    .clone()
                    .map_or(None, |storage_engine| Some(storage_engine.0)),
            )
            .unique(self.unique)
            .version(self.version.map_or(None, |version| {
                Some(match version {
                    0 => mongodb::options::IndexVersion::V0,
                    1 => mongodb::options::IndexVersion::V1,
                    2 => mongodb::options::IndexVersion::V2,
                    _custom => mongodb::options::IndexVersion::Custom(_custom),
                })
            }))
            .default_language(self.default_language.clone())
            .language_override(self.language_override.clone())
            .text_index_version(self.text_index_version.map_or(None, |version| {
                Some(match version {
                    1 => mongodb::options::TextIndexVersion::V1,
                    2 => mongodb::options::TextIndexVersion::V2,
                    3 => mongodb::options::TextIndexVersion::V3,
                    _custom => mongodb::options::TextIndexVersion::Custom(_custom),
                })
            }))
            .weights(self.weights.clone().map_or(None, |weights| Some(weights.0)))
            .sphere_2d_index_version(self.sphere_2d_index_version.map_or(None, |version| {
                Some(match version {
                    2 => mongodb::options::Sphere2DIndexVersion::V2,
                    3 => mongodb::options::Sphere2DIndexVersion::V3,
                    _custom => mongodb::options::Sphere2DIndexVersion::Custom(_custom),
                })
            }))
            .bits(self.bits)
            .max(self.max)
            .min(self.min)
            .bucket_size(self.bucket_size)
            .partial_filter_expression(
                self.partial_filter_expression
                    .clone()
                    .map_or(None, |partial_filter_expression| {
                        Some(partial_filter_expression.0)
                    }),
            )
            .wildcard_projection(
                self.wildcard_projection
                    .clone()
                    .map_or(None, |wildcard_projection| Some(wildcard_projection.0)),
            )
            .hidden(self.hidden)
            .build()
    }
}
