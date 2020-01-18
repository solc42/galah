use checkm::GenomeQuality;
use finch::distance::distance;
use finch::serialization::Sketch;
use rayon::prelude::*;

pub fn print_metaani_distances(
    genome_fasta_paths: &[&str],
    genome_qualities: &[GenomeQuality],
    n_hashes: usize,
    kmer_length: u8,
) {
    // Generate sketches for all input files
    let mut filter = finch::filtering::FilterParams { // dummy, no filtering is applied.
        filter_on: None,
        abun_filter: (None, None),
        err_filter: 0f32,
        strand_filter: 0f32,
    };
    info!("Sketching genomes for clustering ..");
    let sketches = finch::mash_files(
        genome_fasta_paths,
        n_hashes,
        n_hashes,
        kmer_length,
        &mut filter,
        true,
        0)
        .expect("Failed to create finch sketches for input genomes");
    info!("Finished sketching genomes.");

    // Print distances
    info!("Computing and printing ANI values ..");

    println!("\
        genome1\t\
        genome2\t\
        metani\t\
        finch_dist\t\
        completeness_corrected_finch_ani\t\
        genome1_length\t\
        genome2_length\t\
        genome1_completeness\t\
        genome1_contamination\t\
        genome2_completeness\t\
        genome2_contamination");

    genome_fasta_paths.into_par_iter().enumerate().for_each(|(i, genome1_fasta_path)| {
        for j in (i+1)..genome_fasta_paths.len() {
            // TODO: Finch distance calculated twice, which is not necessary.
            let finch = distance(&sketches.sketches[i].hashes, &sketches.sketches[j].hashes, "", "", true)
                .expect("Failed to calculate finch distance");

            let corrected = 1.0136269*(1.0-finch.mashDistance)*100. -
                (1.3970929*(genome_qualities[i].completeness*genome_qualities[j].completeness) as f64);
            
            println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                genome1_fasta_path,
                genome_fasta_paths[j],
                (1.0-metaani_dist(
                    &sketches.sketches[i],
                    &sketches.sketches[j],
                    &genome_qualities[i],
                    &genome_qualities[j],
                    kmer_length
                ))*100.,
                finch.mashDistance,
                corrected,
                &sketches.sketches[i].seqLength.unwrap(),
                &sketches.sketches[j].seqLength.unwrap(),
                &genome_qualities[i].completeness,
                &genome_qualities[i].contamination,
                &genome_qualities[j].completeness,
                &genome_qualities[j].contamination,
            );
        }
    })
}

// pub fn metaani_clusters(
//     genome_fasta_paths: &[&str],
//     genome_qualities: &GenomeQuality,
//     ani: f32,
//     n_hashes: usize,
//     kmer_length: u8,
// ) -> Vec<Vec<usize>> {
//     panic!("Unimplemented ATM");

//     // Generate sketches for all input files
//     let mut filter = finch::filtering::FilterParams { // dummy, no filtering is applied.
//         filter_on: None,
//         abun_filter: (None, None),
//         err_filter: 0f32,
//         strand_filter: 0f32,
//     };
//     info!("Sketching genomes for clustering ..");
//     let sketches = finch::mash_files(
//         genomes,
//         n_hashes,
//         n_hashes,
//         kmer_length,
//         &mut filter,
//         true,
//         0)
//         .expect("Failed to create finch sketches for input genomes");
//     info!("Finished sketching genomes for clustering.");

//     // Greedily find reps
//     info!("Finding cluster representatives ..");
//     let clusters = find_metani_representatives(&sketches.sketches, &genome_qualities, ani);
    
//     // Reassign non-reps based so they are assigned to the nearest
//     // representative.
//     info!("Assigning genomes to representatives ..");
//     return find_metani_memberships(&clusters, &sketches.sketches);
// }

// fn find_metani_representatives
//     sketches: &[Sketch],
//     genome_qualities: &GenomeQuality,
//     ani_threshold: f64)
// -> BTreeSet<usize> {

// }

fn metani_corrected_jaccard_distance(
    sketch1: &Sketch,
    sketch2: &Sketch,
    quality1: &GenomeQuality,
    quality2: &GenomeQuality,
) -> f32 {
    let finch = distance(&sketch1.hashes, &sketch2.hashes, "", "", true)
        .expect("Failed to calculate distance by sketch comparison");

    // let comp1 = quality1.completeness;
    // let comp2 = quality2.completeness;

    // let cont_fix1 = comp1 / (comp1 + quality1.contamination);
    // let cont_fix2 = comp2 / (comp2 + quality2.contamination);

    let attempt1 = finch.commonHashes as f32 / finch.totalHashes as f32
        / quality1.completeness / quality2.completeness;
        // / cont_fix1 / cont_fix2;
    if attempt1 > 1.0 {
        1.0
    } else {
        attempt1
    }
}

fn metaani_dist(
    sketch1: &Sketch,
    sketch2: &Sketch,
    quality1: &GenomeQuality,
    quality2: &GenomeQuality,
    kmer_length: u8,
) -> f32 {
    let corrected_jaccard = metani_corrected_jaccard_distance(
        sketch1, sketch2, quality1, quality2
    );
    -1.0 / kmer_length as f32 * ((2.*corrected_jaccard / (1.+corrected_jaccard)).ln())
}

// /// For each genome (sketch) assign it to the closest representative genome:
// fn find_minhash_memberships(
//     representatives: &BTreeSet<usize>,
//     sketches: &[Sketch],
// ) -> Vec<Vec<usize>> {
// }