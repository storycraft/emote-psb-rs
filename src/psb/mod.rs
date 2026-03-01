pub mod error;
pub mod table;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek};

use crate::{
    PSB_SIGNATURE,
    psb::{
        error::PsbOpenError,
        table::{PsbResourceTable, PsbStringTable},
    },
    value::binary_tree::PsbBinaryTree,
};

#[derive(Debug, Clone)]
pub struct PsbFile {
    pub encrypted: bool,
    pub version: u16,

    pub names: Vec<String>,
    pub strings: PsbStringTable,
    pub resources: PsbResourceTable,

    /// Offset to root object
    pub entrypoint: u32,

    pub checksum: Option<u32>,
    pub extra: Option<PsbResourceTable>,
}

impl PsbFile {
    /// Open Psb file from stream
    pub async fn open<T: AsyncRead + AsyncSeek + Unpin>(
        mut stream: T,
    ) -> Result<Self, PsbOpenError> {
        let signature = stream.read_u32_le().await?;
        if signature != PSB_SIGNATURE {
            return Err(PsbOpenError::InvalidSignature);
        }

        let version = stream.read_u16_le().await?;
        let encrypted = stream.read_u16_le().await? != 0;

        let _ = stream.read_u32_le().await?;

        let name_offset = stream.read_u32_le().await?;

        let string_offset = stream.read_u32_le().await?;
        let string_data_start = stream.read_u32_le().await?;

        let resource_offset = stream.read_u32_le().await?;
        let resource_lengths = stream.read_u32_le().await?;
        let resource_data_start = stream.read_u32_le().await?;

        let entrypoint = stream.read_u32_le().await?;

        let checksum = if version > 2 {
            Some(stream.read_u32_le().await?)
        } else {
            None
        };

        let extra = if version > 3 {
            let extra_resource_offset = stream.read_u32_le().await?;
            let extra_resource_lengths = stream.read_u32_le().await?;
            let extra_resource_data_start = stream.read_u32_le().await?;

            Some(
                PsbResourceTable::read_io(
                    &mut stream,
                    extra_resource_offset,
                    extra_resource_lengths,
                    extra_resource_data_start,
                )
                .await
                .map_err(PsbOpenError::Resources)?,
            )
        } else {
            None
        };

        let names = PsbBinaryTree::read_io(&mut stream, name_offset)
            .await
            .map_err(PsbOpenError::Names)?
            .list
            .into_iter()
            .map(|raw_name| String::from_utf8_lossy(&raw_name).into_owned())
            .collect::<Vec<_>>();

        let strings = PsbStringTable::read_io(&mut stream, string_offset, string_data_start)
            .await
            .map_err(PsbOpenError::Strings)?;

        let resources = PsbResourceTable::read_io(
            &mut stream,
            resource_offset,
            resource_lengths,
            resource_data_start,
        )
        .await
        .map_err(PsbOpenError::Resources)?;

        Ok(Self {
            encrypted,
            version,
            names,
            strings,
            resources,
            entrypoint,
            checksum,
            extra,
        })
    }
}
