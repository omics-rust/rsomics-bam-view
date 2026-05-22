use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use noodles::bam;
use noodles::bgzf;
use noodles::sam;
use rsomics_common::{Result, RsomicsError};

#[derive(Debug, Clone, Default)]
pub struct ViewFilter {
    pub require_flags: u16,
    pub exclude_flags: u16,
    pub min_mapq: u8,
    pub count_only: bool,
}

fn passes(flags: sam::alignment::record::Flags, mapq: Option<u8>, f: &ViewFilter) -> bool {
    let bits = flags.bits();
    if f.require_flags != 0 && (bits & f.require_flags) != f.require_flags {
        return false;
    }
    if f.exclude_flags != 0 && (bits & f.exclude_flags) != 0 {
        return false;
    }
    if f.min_mapq > 0 && mapq.unwrap_or(0) < f.min_mapq {
        return false;
    }
    true
}

pub fn view_bam(
    input: &Path,
    output_path: Option<&Path>,
    filter: &ViewFilter,
    workers: NonZero<usize>,
) -> Result<u64> {
    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    if filter.count_only {
        return count_bam(&mut reader, filter);
    }

    // File output uses bamio's multithreaded BGZF writer (libdeflate); stdout
    // falls back to the single-threaded writer.
    match output_path {
        Some(path) => {
            let mut writer = rsomics_bamio::create_with_workers(path, workers)?;
            write_filtered(&mut reader, &mut writer, &header, filter)
        }
        None => {
            let mut writer = bam::io::Writer::new(std::io::stdout().lock());
            write_filtered(&mut reader, &mut writer, &header, filter)
        }
    }
}

fn write_filtered<W: Write>(
    reader: &mut rsomics_bamio::ParallelBamReader,
    writer: &mut bam::io::Writer<W>,
    header: &sam::Header,
    filter: &ViewFilter,
) -> Result<u64> {
    writer.write_header(header).map_err(RsomicsError::Io)?;

    let mut count: u64 = 0;
    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();
        let mapq = record.mapping_quality().map(|q| q.get());

        if !passes(flags, mapq, filter) {
            continue;
        }
        count += 1;
        writer
            .write_record(header, &record)
            .map_err(RsomicsError::Io)?;
    }
    Ok(count)
}

fn count_bam<R>(reader: &mut bam::io::Reader<R>, filter: &ViewFilter) -> Result<u64>
where
    R: bgzf::io::BufRead + bgzf::io::Seek,
{
    let mut count: u64 = 0;
    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let flags = record.flags();
        let mapq = record.mapping_quality().map(|q| q.get());
        if passes(flags, mapq, filter) {
            count += 1;
        }
    }
    Ok(count)
}
