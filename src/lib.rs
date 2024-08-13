pub use hoplite_verbs_rs::*;
use rand::prelude::SliceRandom;
use std::collections::HashSet;

pub trait GetRandom {
    fn change_params(
        &mut self,
        n_params_to_change: u8,
        parameters: &VerbParameters,
        params_do_not_change: &mut [HcParameters],
    ) -> Vec<HcParameters>;
    fn random_form(
        &self,
        num_changes: u8,
        highest_unit: Option<i16>,
        parameters: &VerbParameters,
        filter_forms: Option<&HashSet<u32>>,
    ) -> (HcGreekVerbForm, Diagnostics);
    fn block_for_hq_unit(&self, unit: Option<i16>) -> bool;
    fn block_middle_passive(&self, new_form: &HcGreekVerbForm) -> bool;
    fn param_hash(&self) -> u32;
    fn extract_params_from_hash(&mut self, value: u32);
}

impl GetRandom for HcGreekVerbForm {
    // add param for top unit
    fn random_form(
        &self,
        num_changes: u8,
        highest_unit: Option<i16>,
        parameters: &VerbParameters,
        filter_forms: Option<&HashSet<u32>>, //previously used forms we don't want to return
    ) -> (HcGreekVerbForm, Diagnostics) {
        let mut pf: HcGreekVerbForm;
        let mut num_skipped = 0;
        let mut ignore_filter = false;

        let mut diag = Diagnostics {
            dash: 0,
            middle_passive: 0,
            blocked_for_unit: 0,
            filtered: 0,
            illegal: 0,
        };

        loop {
            pf = self.clone();
            pf.change_params(
                num_changes,
                parameters,
                &mut [], //HcParameters::Person, HcParameters::Number
            );
            let vf = pf.get_form(false);
            if num_skipped > 2000 {
                // println!(
                //     "AAABBB error: {}",
                //     if filter_forms.is_some() {
                //         filter_forms.unwrap().len()
                //     } else {
                //         222
                //     }
                // );
                //error!("random form 2000 cycles");
                ignore_filter = true;
            } else if num_skipped > 4000 {
                // println!(
                //     "AAABBB2 error: {}",
                //     if filter_forms.is_some() {
                //         filter_forms.unwrap().len()
                //     } else {
                //         222
                //     }
                // );
                //error!("random form 4000 cycles");
                break;
            }
            num_skipped += 1;
            match vf {
                Ok(res) => {
                    if res.last().unwrap().form == "—"
                        || self.block_middle_passive(&pf)
                        || pf.block_for_hq_unit(highest_unit)
                        || (filter_forms.is_some()
                            && !ignore_filter
                            && filter_forms.unwrap().contains(&pf.param_hash()))
                    {
                        if self.block_middle_passive(&pf) {
                            diag.middle_passive += 1;
                        } else if pf.block_for_hq_unit(highest_unit) {
                            diag.blocked_for_unit += 1;
                        } else if filter_forms.is_some()
                            && !ignore_filter
                            && filter_forms.unwrap().contains(&pf.param_hash())
                        {
                            diag.filtered += 1;
                        } else if res.last().unwrap().form == "—" {
                            diag.dash += 1;
                        }

                        let _reason = if self.block_middle_passive(&pf) {
                            String::from("middle/passive just used")
                        } else if pf.block_for_hq_unit(highest_unit) {
                            format!("not in unit: {:?}", highest_unit)
                        } else if filter_forms.is_some()
                            && !ignore_filter
                            && filter_forms.unwrap().contains(&pf.param_hash())
                        {
                            "already used".to_string()
                        } else if res.last().unwrap().form == "—" {
                            format!("block bad form {:?}", res)
                        } else {
                            String::from("unknown reason")
                        };
                        // println!(
                        //     "\t{}: {:?} {:?}",
                        //     num_skipped,
                        //     res.last().unwrap().form,
                        //     reason
                        // );

                        continue;
                    } else {
                        //println!("{}", res.last().unwrap().form);
                        break;
                    }
                } //only 3rd pl consonant stem perfects/pluperfects return - now
                Err(_e) => {
                    diag.illegal += 1;
                    //println!("\t{}: {:?}", num_skipped, e);
                    continue;
                }
            }
        }
        (pf, diag)
    }

    // num params to change must be equal or less than num params with more than one value
    // params_do_not_change: pass in params from last change, so we don't change the same ones again
    fn change_params(
        &mut self,
        n_params_to_change: u8,
        parameters: &VerbParameters,
        params_do_not_change: &mut [HcParameters],
    ) -> Vec<HcParameters> {
        let mut possible_params = vec![
            HcParameters::Person,
            HcParameters::Number,
            HcParameters::Tense,
            HcParameters::Mood,
            HcParameters::Voice,
        ];

        if parameters.persons.len() == 1 {
            self.person = Some(parameters.persons[0]);
            possible_params.retain(|e| *e != HcParameters::Person);
        }
        if parameters.numbers.len() == 1 {
            self.number = Some(parameters.numbers[0]);
            possible_params.retain(|e| *e != HcParameters::Number);
        }
        if parameters.tenses.len() == 1 {
            self.tense = parameters.tenses[0];
            possible_params.retain(|e| *e != HcParameters::Tense);
        }
        if parameters.moods.len() == 1 {
            self.mood = parameters.moods[0];
            possible_params.retain(|e| *e != HcParameters::Mood);
        }
        if parameters.voices.len() == 1 {
            self.voice = parameters.voices[0];
            possible_params.retain(|e| *e != HcParameters::Voice);
        }

        if self.person.is_none() || self.number.is_none() || possible_params.is_empty() {
            return vec![];
        }

        let mut rng = rand::thread_rng();

        if !params_do_not_change.is_empty() && possible_params.len() > 1 {
            params_do_not_change.shuffle(&mut rng); //shuffle, so not always first param
            if let Some(aa) = params_do_not_change.first() {
                possible_params.retain(|e| *e != *aa);
            }
        }

        possible_params.shuffle(&mut rng);
        possible_params.truncate(n_params_to_change.into());

        for p in &possible_params {
            match p {
                HcParameters::Person => {
                    self.person = Some(
                        **parameters
                            .persons
                            .iter()
                            .filter(|x| **x != self.person.unwrap())
                            .collect::<Vec<_>>()
                            .choose(&mut rand::thread_rng())
                            .unwrap(),
                    );
                }
                HcParameters::Number => {
                    self.number = Some(
                        **parameters
                            .numbers
                            .iter()
                            .filter(|x| **x != self.number.unwrap())
                            .collect::<Vec<_>>()
                            .choose(&mut rand::thread_rng())
                            .unwrap(),
                    );
                }
                HcParameters::Tense => {
                    self.tense = **parameters
                        .tenses
                        .iter()
                        .filter(|x| **x != self.tense)
                        .collect::<Vec<_>>()
                        .choose(&mut rand::thread_rng())
                        .unwrap();
                }
                HcParameters::Voice => {
                    self.voice = **parameters
                        .voices
                        .iter()
                        .filter(|x| **x != self.voice)
                        .collect::<Vec<_>>()
                        .choose(&mut rand::thread_rng())
                        .unwrap();
                }
                HcParameters::Mood => {
                    self.mood = **parameters
                        .moods
                        .iter()
                        .filter(|x| **x != self.mood)
                        .collect::<Vec<_>>()
                        .choose(&mut rand::thread_rng())
                        .unwrap();
                }
            }
        }
        possible_params
    }

    // if middle or passive do not change voice to passive or middle unless tense is aorist or future
    // true to block change, false to allow change
    // AND before OR
    fn block_middle_passive(&self, new_form: &HcGreekVerbForm) -> bool {
        (self.voice == HcVoice::Middle && new_form.voice == HcVoice::Passive
            || self.voice == HcVoice::Passive && new_form.voice == HcVoice::Middle)
            && new_form.tense != HcTense::Aorist
            && new_form.tense != HcTense::Future
            && self.tense != HcTense::Aorist
            && self.tense != HcTense::Future
    }

    fn block_for_hq_unit(&self, unit: Option<i16>) -> bool {
        match unit {
            Some(unit) => {
                let is_mi_verb = self.verb.pps[0].ends_with("μι");
                let is_isthmi = self.verb.pps[0].ends_with("στημι");

                let is_isthmi_perf = is_isthmi
                    && (self.tense == HcTense::Aorist
                        || self.tense == HcTense::Perfect
                        || self.tense == HcTense::Pluperfect);

                let is_future_optative =
                    self.tense == HcTense::Future && self.mood == HcMood::Optative;

                let is_consonant_stem_third_plural = self.is_consonant_stem("")
                    && (self.tense == HcTense::Perfect || self.tense == HcTense::Pluperfect)
                    && (self.voice == HcVoice::Middle || self.voice == HcVoice::Passive)
                    && self.person == Some(HcPerson::Third)
                    && self.number == Some(HcNumber::Plural);

                if unit <= 2 {
                    //2 and under active indicative and not perfect or pluperfect
                    if self.tense == HcTense::Perfect
                        || self.tense == HcTense::Pluperfect
                        || self.voice != HcVoice::Active
                        || self.mood != HcMood::Indicative
                        || is_mi_verb
                    {
                        return true;
                    }
                } else if unit <= 4 {
                    //4 and under must be active, no imperatives, no future optative
                    if self.voice != HcVoice::Active
                        || self.mood == HcMood::Imperative
                        || is_mi_verb
                        || is_future_optative
                    {
                        return true;
                    }
                } else if unit <= 6 {
                    //6 and under can't be middle, no imperatives, no future optative
                    if self.voice == HcVoice::Middle
                        || self.mood == HcMood::Imperative
                        || is_mi_verb
                        || is_future_optative
                        || is_consonant_stem_third_plural
                    {
                        return true;
                    }
                } else if unit <= 10 {
                    //10 and under no imperatives, no future optative
                    if self.mood == HcMood::Imperative
                        || is_mi_verb
                        || is_future_optative
                        || is_consonant_stem_third_plural
                    {
                        return true;
                    }
                } else if unit <= 11 {
                    //11 and under no aorists of mi verbs, no perf/plup of isthmi, no future optative
                    if is_mi_verb || is_future_optative || is_consonant_stem_third_plural {
                        return true;
                    }
                } else if unit <= 12 {
                    //12 and under no aorists of mi verbs, no perf/plup of isthmi, no future optative
                    if (is_mi_verb && self.tense == HcTense::Aorist)
                        || is_isthmi_perf
                        || is_future_optative
                        || is_consonant_stem_third_plural
                    {
                        return true;
                    }
                    // todo deiknumi verbs?
                } else if unit <= 15 {
                    //15 and under no future optative
                    if is_future_optative || is_consonant_stem_third_plural {
                        return true;
                    }
                } else if unit <= 19 {
                    //19 and under no 3rd plural of consonant stem perf/plup mid/pass
                    if is_consonant_stem_third_plural {
                        return true;
                    }
                }
                false
            }
            None => false,
        }
    }

    //only call on finite verbs, maybe change to return Option<u32> to handle non-finites?
    fn param_hash(&self) -> u32 {
        let p_count = 3;
        let n_count = 2;
        let t_count = 6;
        let m_count = 4;

        let voice = self.voice.to_i16();
        let mood = self.mood.to_i16();
        let tense = self.tense.to_i16();
        let number = if self.number.is_some() {
            self.number.unwrap().to_i16()
        } else {
            2 //panic!() //add an extra number, in case of None: it just has to be unique
        };
        let person = if self.person.is_some() {
            self.person.unwrap().to_i16()
        } else {
            3 //panic!() //add an extra number, in case of None: it just has to be unique
        };

        //calculate unique hash from param values
        (voice * m_count * t_count * n_count * p_count
            + mood * t_count * n_count * p_count
            + tense * n_count * p_count
            + number * p_count
            + person)
            .try_into()
            .unwrap()
    }

    fn extract_params_from_hash(&mut self, value: u32) {
        let p_count = 3;
        let n_count = 2;
        let t_count = 6;
        let m_count = 4;

        let voice = value / (m_count * t_count * n_count * p_count);
        let remaining = value % (m_count * t_count * n_count * p_count);

        let mood = remaining / (t_count * n_count * p_count);
        let remaining = remaining % (t_count * n_count * p_count);

        let tense = remaining / (n_count * p_count);
        let remaining = remaining % (n_count * p_count);

        let number = remaining / p_count;
        let person = remaining % p_count;

        self.person = Some(HcPerson::from_i16(person.try_into().unwrap()));
        self.number = Some(HcNumber::from_i16(number.try_into().unwrap()));
        self.tense = HcTense::from_i16(tense.try_into().unwrap());
        self.mood = HcMood::from_i16(mood.try_into().unwrap());
        self.voice = HcVoice::from_i16(voice.try_into().unwrap());

        //(person, number, tense, mood, voice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_param_hash() {
        let luw = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
        let verb = Arc::new(HcGreekVerb::from_string(1, luw, REGULAR, 0).unwrap());
        let a = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };

        //b has different params from a
        let mut b = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::Third),
            number: Some(HcNumber::Plural),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Imperative,
            gender: None,
            case: None,
        };

        let hash = a.param_hash();
        b.extract_params_from_hash(hash); //this should set b's params to equal a: thus the forms are equal
                                          //test round trip to param hash to form again
        assert_eq!(a, b);
    }

    /*
        #[test]
        fn test_oida() {
            // All elements can be initialized to the same value.
            let mut results: [usize; 432] = [0; 432];

            let oida = "οἶδα, εἴσομαι, —, —, —, —";
            let verb = Arc::new(HcGreekVerb::from_string(1, oida, REGULAR, 0).unwrap());
            let a = HcGreekVerbForm {
                verb: verb.clone(),
                person: Some(HcPerson::First),
                number: Some(HcNumber::Singular),
                tense: HcTense::Perfect,
                voice: HcVoice::Active,
                mood: HcMood::Indicative,
                gender: None,
                case: None,
            };

            let first_hash = a.param_hash() as usize;
            let mut idx: usize = 0;

            let max_changes = 2;
            let highest_unit = 20;
            let verb_params = VerbParameters {
                persons: vec![HcPerson::First, HcPerson::Second, HcPerson::Third],
                numbers: vec![HcNumber::Singular, HcNumber::Plural],
                tenses: vec![
                    HcTense::Present,
                    HcTense::Imperfect,
                    HcTense::Future,
                    HcTense::Aorist,
                    HcTense::Perfect,
                    HcTense::Pluperfect,
                ],
                voices: vec![HcVoice::Active, HcVoice::Middle, HcVoice::Passive],
                moods: vec![
                    HcMood::Indicative,
                    HcMood::Subjunctive,
                    HcMood::Optative,
                    HcMood::Imperative,
                ],
            };

            //let mut form_filter:HashSet<u32> = HashSet::new();
            // form_filter.insert(b.param_hash());
            // form_filter.insert(c.param_hash());

            let count = 100_000;

            for i in 0..count {
                let (a, _diag) = a.random_form(max_changes, Some(highest_unit), &verb_params, None);
                println!(
                    "{} {}",
                    a.param_hash(),
                    a.get_form(false).unwrap().last().unwrap().form
                );
                idx = a.param_hash() as usize;
                results[idx] += 1;

                // if i % 10 == 0 {
                //     form_filter.clear();
                // }
                //form_filter.insert(d.param_hash());
                //assert!(!form_filter.contains(&d.param_hash()));
                //assert_ne!(d.param_hash(), c.param_hash()); //the random form should never equal c because c was added to filter HashSet
            }
            //assert_eq!(0, results[0]);

            let mut b = HcGreekVerbForm {
                verb: verb.clone(),
                person: Some(HcPerson::Second),
                number: Some(HcNumber::Singular),
                tense: HcTense::Perfect,
                voice: HcVoice::Active,
                mood: HcMood::Indicative,
                gender: None,
                case: None,
            };
            let m: Vec<(usize, f64)> = results
                .iter()
                .enumerate()
                .map(|(i, e)| (i, (*e as f64 / count as f64)))
                .filter(|e| e.1 != 0.0)
                .collect();

            //println!("count {} {:?}", results[b.param_hash() as usize], m);
            for i in m {
                b.extract_params_from_hash(i.0.try_into().unwrap());
                println!("{} {:?}", i.0, b);
            }
            assert!(
                results[b.param_hash() as usize] / count > 30 / count
                    && results[b.param_hash() as usize] / count < 34 / count,
                "b.param_hash() {}",
                results[b.param_hash() as usize] / count
            );
        }
    */

    #[test]
    fn test_random2() {
        let luw = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
        let verb = Arc::new(HcGreekVerb::from_string(1, luw, REGULAR, 0).unwrap());
        let a = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };

        let max_changes = 2;
        let highest_unit = 2;
        let verb_params = VerbParameters {
            persons: vec![HcPerson::First, HcPerson::Second, HcPerson::Third],
            numbers: vec![HcNumber::Singular, HcNumber::Plural],
            tenses: vec![
                HcTense::Present,
                HcTense::Imperfect,
                HcTense::Future,
                HcTense::Aorist,
                HcTense::Perfect,
                HcTense::Pluperfect,
            ],
            voices: vec![HcVoice::Active, HcVoice::Middle, HcVoice::Passive],
            moods: vec![
                HcMood::Indicative,
                HcMood::Subjunctive,
                HcMood::Optative,
                HcMood::Imperative,
            ],
        };

        let mut form_filter = HashSet::new();
        // form_filter.insert(b.param_hash());
        // form_filter.insert(c.param_hash());

        for _i in 0..10 {
            let (d, _diag) = a.random_form(
                max_changes,
                Some(highest_unit),
                &verb_params,
                Some(&form_filter),
            );
            form_filter.insert(d.param_hash());
            //assert!(!form_filter.contains(&d.param_hash()));
            //assert_ne!(d.param_hash(), c.param_hash()); //the random form should never equal c because c was added to filter HashSet
        }
    }

    #[test]
    fn test_random() {
        let luw = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
        let verb = Arc::new(HcGreekVerb::from_string(1, luw, REGULAR, 0).unwrap());
        let a = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::Second),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let c = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::Third),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };

        let max_changes = 1;
        let highest_unit = 2;
        let verb_params = VerbParameters {
            persons: vec![HcPerson::First, HcPerson::Second, HcPerson::Third],
            numbers: vec![HcNumber::Singular, HcNumber::Plural],
            tenses: vec![HcTense::Present, HcTense::Imperfect],
            voices: vec![HcVoice::Active],
            moods: vec![HcMood::Indicative],
        };

        let mut form_filter = HashSet::new();
        form_filter.insert(b.param_hash());
        form_filter.insert(c.param_hash());

        for _i in 0..10_000 {
            let (d, _diag) = a.random_form(
                max_changes,
                Some(highest_unit),
                &verb_params,
                Some(&form_filter),
            );
            assert!(!form_filter.contains(&d.param_hash()));
            assert_ne!(d.param_hash(), c.param_hash()); //the random form should never equal c because c was added to filter HashSet
        }
    }

    #[test]
    fn test_change_param_block_last_param_change() {
        let luw = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
        let verb = Arc::new(HcGreekVerb::from_string(1, luw, REGULAR, 0).unwrap());
        let mut a = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };

        let num_changes = 2;
        let parameters = VerbParameters {
            persons: vec![HcPerson::First, HcPerson::Second, HcPerson::Third],
            numbers: vec![HcNumber::Singular, HcNumber::Plural],
            tenses: vec![
                HcTense::Present,
                HcTense::Imperfect,
                HcTense::Future,
                HcTense::Aorist,
                HcTense::Perfect,
                HcTense::Pluperfect,
            ],
            voices: vec![HcVoice::Active, HcVoice::Middle, HcVoice::Passive],
            moods: vec![
                HcMood::Indicative,
                HcMood::Subjunctive,
                HcMood::Optative,
                HcMood::Imperative,
            ],
        };

        let count = 10_000;
        for _i in 0..count {
            a.change_params(num_changes, &parameters, &mut [HcParameters::Tense]);
            assert_eq!(a.tense, HcTense::Present); //don't change tense if tense is passed in above
        }
    }

    #[test]
    fn test_random_param_change_distribution() {
        let mut persons = [0, 0, 0];
        let mut numbers = [0, 0];
        let mut tenses = [0, 0, 0, 0, 0, 0];
        let mut moods = [0, 0, 0, 0];
        let mut voices = [0, 0, 0];
        let mut param_hash = 0;

        let luw = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
        let verb = Arc::new(HcGreekVerb::from_string(1, luw, REGULAR, 0).unwrap());
        let mut a = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };

        let num_changes = 2;
        let parameters = VerbParameters {
            persons: vec![HcPerson::First, HcPerson::Second, HcPerson::Third],
            numbers: vec![HcNumber::Singular, HcNumber::Plural],
            tenses: vec![
                HcTense::Present,
                HcTense::Imperfect,
                HcTense::Future,
                HcTense::Aorist,
                HcTense::Perfect,
                HcTense::Pluperfect,
            ],
            voices: vec![HcVoice::Active, HcVoice::Middle, HcVoice::Passive],
            moods: vec![
                HcMood::Indicative,
                HcMood::Subjunctive,
                HcMood::Optative,
                HcMood::Imperative,
            ],
        };

        let count = 100_000;
        for _i in 0..count {
            a.change_params(num_changes, &parameters, &mut []);
            persons[a.person.unwrap().to_i16() as usize] += 1;
            numbers[a.number.unwrap().to_i16() as usize] += 1;
            tenses[a.tense.to_i16() as usize] += 1;
            moods[a.mood.to_i16() as usize] += 1;
            voices[a.voice.to_i16() as usize] += 1;

            param_hash += a.param_hash();
        }
        //sum of hash divided by count should be half of total number of possible forms (432 = 216)
        assert!(
            (param_hash as f64 / count as f64) > 214.0
                && (param_hash as f64 / count as f64) < 218.0
        );

        //check distribution of each param:
        assert!(
            (persons[0] as f64 / count as f64) > 0.31 && (persons[0] as f64 / count as f64) < 0.35
        );
        assert!(
            (persons[1] as f64 / count as f64) > 0.31 && (persons[1] as f64 / count as f64) < 0.35
        );
        assert!(
            (persons[2] as f64 / count as f64) > 0.31 && (persons[2] as f64 / count as f64) < 0.35
        );

        assert!(
            (numbers[0] as f64 / count as f64) > 0.48 && (numbers[0] as f64 / count as f64) < 0.52
        );
        assert!(
            (numbers[1] as f64 / count as f64) > 0.48 && (numbers[1] as f64 / count as f64) < 0.52
        );

        assert!(
            (tenses[0] as f64 / count as f64) > 0.14 && (tenses[0] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[1] as f64 / count as f64) > 0.14 && (tenses[1] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[2] as f64 / count as f64) > 0.14 && (tenses[2] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[3] as f64 / count as f64) > 0.14 && (tenses[3] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[4] as f64 / count as f64) > 0.14 && (tenses[4] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[5] as f64 / count as f64) > 0.14 && (tenses[5] as f64 / count as f64) < 0.18
        );

        assert!((moods[0] as f64 / count as f64) > 0.23 && (moods[0] as f64 / count as f64) < 0.27);
        assert!((moods[1] as f64 / count as f64) > 0.23 && (moods[1] as f64 / count as f64) < 0.27);
        assert!((moods[2] as f64 / count as f64) > 0.23 && (moods[2] as f64 / count as f64) < 0.27);
        assert!((moods[3] as f64 / count as f64) > 0.23 && (moods[3] as f64 / count as f64) < 0.27);

        assert!(
            (voices[0] as f64 / count as f64) > 0.31 && (voices[0] as f64 / count as f64) < 0.35
        );
        assert!(
            (voices[1] as f64 / count as f64) > 0.31 && (voices[1] as f64 / count as f64) < 0.35
        );
        assert!(
            (voices[2] as f64 / count as f64) > 0.31 && (voices[2] as f64 / count as f64) < 0.35
        );
    }

    //same as above, but with oida = same results
    #[test]
    fn test_random_param_change_distribution_oida() {
        let mut persons = [0, 0, 0];
        let mut numbers = [0, 0];
        let mut tenses = [0, 0, 0, 0, 0, 0];
        let mut moods = [0, 0, 0, 0];
        let mut voices = [0, 0, 0];
        let mut param_hash = 0;

        let oida = "οἶδα, εἴσομαι, —, —, —, —";
        let verb = Arc::new(HcGreekVerb::from_string(1, oida, REGULAR, 0).unwrap());
        let mut a = HcGreekVerbForm {
            verb: verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };

        let num_changes = 2;
        let parameters = VerbParameters {
            persons: vec![HcPerson::First, HcPerson::Second, HcPerson::Third],
            numbers: vec![HcNumber::Singular, HcNumber::Plural],
            tenses: vec![
                HcTense::Present,
                HcTense::Imperfect,
                HcTense::Future,
                HcTense::Aorist,
                HcTense::Perfect,
                HcTense::Pluperfect,
            ],
            voices: vec![HcVoice::Active, HcVoice::Middle, HcVoice::Passive],
            moods: vec![
                HcMood::Indicative,
                HcMood::Subjunctive,
                HcMood::Optative,
                HcMood::Imperative,
            ],
        };

        let count = 100_000;
        for _i in 0..count {
            a.change_params(num_changes, &parameters, &mut []);
            persons[a.person.unwrap().to_i16() as usize] += 1;
            numbers[a.number.unwrap().to_i16() as usize] += 1;
            tenses[a.tense.to_i16() as usize] += 1;
            moods[a.mood.to_i16() as usize] += 1;
            voices[a.voice.to_i16() as usize] += 1;

            param_hash += a.param_hash();
        }
        //sum of hash divided by count should be half of total number of possible forms (432 = 216)
        assert!(
            (param_hash as f64 / count as f64) > 214.0
                && (param_hash as f64 / count as f64) < 218.0
        );

        //check distribution of each param:
        assert!(
            (persons[0] as f64 / count as f64) > 0.31 && (persons[0] as f64 / count as f64) < 0.35
        );
        assert!(
            (persons[1] as f64 / count as f64) > 0.31 && (persons[1] as f64 / count as f64) < 0.35
        );
        assert!(
            (persons[2] as f64 / count as f64) > 0.31 && (persons[2] as f64 / count as f64) < 0.35
        );

        assert!(
            (numbers[0] as f64 / count as f64) > 0.48 && (numbers[0] as f64 / count as f64) < 0.52
        );
        assert!(
            (numbers[1] as f64 / count as f64) > 0.48 && (numbers[1] as f64 / count as f64) < 0.52
        );

        assert!(
            (tenses[0] as f64 / count as f64) > 0.14 && (tenses[0] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[1] as f64 / count as f64) > 0.14 && (tenses[1] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[2] as f64 / count as f64) > 0.14 && (tenses[2] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[3] as f64 / count as f64) > 0.14 && (tenses[3] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[4] as f64 / count as f64) > 0.14 && (tenses[4] as f64 / count as f64) < 0.18
        );
        assert!(
            (tenses[5] as f64 / count as f64) > 0.14 && (tenses[5] as f64 / count as f64) < 0.18
        );

        assert!((moods[0] as f64 / count as f64) > 0.23 && (moods[0] as f64 / count as f64) < 0.27);
        assert!((moods[1] as f64 / count as f64) > 0.23 && (moods[1] as f64 / count as f64) < 0.27);
        assert!((moods[2] as f64 / count as f64) > 0.23 && (moods[2] as f64 / count as f64) < 0.27);
        assert!((moods[3] as f64 / count as f64) > 0.23 && (moods[3] as f64 / count as f64) < 0.27);

        assert!(
            (voices[0] as f64 / count as f64) > 0.31 && (voices[0] as f64 / count as f64) < 0.35
        );
        assert!(
            (voices[1] as f64 / count as f64) > 0.31 && (voices[1] as f64 / count as f64) < 0.35
        );
        assert!(
            (voices[2] as f64 / count as f64) > 0.31 && (voices[2] as f64 / count as f64) < 0.35
        );
    }

    #[test]
    fn block_for_hq_unit() {
        let luw = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
        let luwverb = Arc::new(HcGreekVerb::from_string(1, luw, REGULAR, 0).unwrap());

        let vf = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Perfect,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // block perfects for unit 2
        assert!(vf.block_for_hq_unit(Some(2)));
        // allow perfects for unit 3
        assert!(!vf.block_for_hq_unit(Some(3)));

        // subjunctive/optative in unit 3
        let vf = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Active,
            mood: HcMood::Subjunctive,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(2)));
        assert!(!vf.block_for_hq_unit(Some(3)));

        // subjunctive/optative in unit 3
        let vf = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Active,
            mood: HcMood::Optative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(2)));
        assert!(!vf.block_for_hq_unit(Some(3)));

        // passive voice in unit 5
        let vf = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(4)));
        assert!(!vf.block_for_hq_unit(Some(5)));

        // middle voice in unit 7
        let vf = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(6)));
        assert!(!vf.block_for_hq_unit(Some(7)));

        // imperatives in unit 11
        let vf = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Active,
            mood: HcMood::Imperative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(10)));
        assert!(!vf.block_for_hq_unit(Some(11)));

        // block mi verbs until unit 12
        let isthmi = "ἵστημι, στήσω, ἔστησα / ἔστην, ἕστηκα, ἕσταμαι, ἐστάθην";
        let isthmi_verb = Arc::new(HcGreekVerb::from_string(1, isthmi, REGULAR, 0).unwrap());
        let vf = HcGreekVerbForm {
            verb: isthmi_verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(2)));
        assert!(vf.block_for_hq_unit(Some(3)));
        assert!(vf.block_for_hq_unit(Some(4)));
        assert!(vf.block_for_hq_unit(Some(5)));
        assert!(vf.block_for_hq_unit(Some(6)));
        assert!(vf.block_for_hq_unit(Some(7)));
        assert!(vf.block_for_hq_unit(Some(8)));
        assert!(vf.block_for_hq_unit(Some(9)));
        assert!(vf.block_for_hq_unit(Some(10)));
        assert!(vf.block_for_hq_unit(Some(11)));
        assert!(!vf.block_for_hq_unit(Some(12)));

        // block aorist of mi verbs until unit 13
        let vf = HcGreekVerbForm {
            verb: isthmi_verb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Aorist,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(12)));
        assert!(!vf.block_for_hq_unit(Some(13)));

        // block perfect of isthmi until unit 13
        let vf = HcGreekVerbForm {
            verb: isthmi_verb,
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Perfect,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(12)));
        assert!(!vf.block_for_hq_unit(Some(13)));

        // future optative, not until unit 16
        let vf = HcGreekVerbForm {
            verb: luwverb,
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Active,
            mood: HcMood::Optative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(2)));
        assert!(vf.block_for_hq_unit(Some(3)));
        assert!(vf.block_for_hq_unit(Some(4)));
        assert!(vf.block_for_hq_unit(Some(5)));
        assert!(vf.block_for_hq_unit(Some(6)));
        assert!(vf.block_for_hq_unit(Some(7)));
        assert!(vf.block_for_hq_unit(Some(8)));
        assert!(vf.block_for_hq_unit(Some(9)));
        assert!(vf.block_for_hq_unit(Some(10)));
        assert!(vf.block_for_hq_unit(Some(11)));
        assert!(vf.block_for_hq_unit(Some(12)));
        assert!(vf.block_for_hq_unit(Some(13)));
        assert!(vf.block_for_hq_unit(Some(14)));
        assert!(vf.block_for_hq_unit(Some(15)));
        assert!(!vf.block_for_hq_unit(Some(16)));

        // 3rd plural consonant stem perfects
        let blaptw = "βλάπτω, βλάψω, ἔβλαψα, βέβλαφα, βέβλαμμαι, ἐβλάβην / ἐβλάφθην";
        let cons_stem_verb =
            Arc::new(HcGreekVerb::from_string(1, blaptw, CONSONANT_STEM_PERFECT_PI, 0).unwrap());
        let vf = HcGreekVerbForm {
            verb: cons_stem_verb.clone(),
            person: Some(HcPerson::Third),
            number: Some(HcNumber::Plural),
            tense: HcTense::Perfect,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        assert!(vf.is_consonant_stem(""));

        assert!(vf.block_for_hq_unit(Some(2)));
        assert!(vf.block_for_hq_unit(Some(3)));
        assert!(vf.block_for_hq_unit(Some(4)));
        assert!(vf.block_for_hq_unit(Some(5)));
        assert!(vf.block_for_hq_unit(Some(6)));
        assert!(vf.block_for_hq_unit(Some(7)));
        assert!(vf.block_for_hq_unit(Some(8)));
        assert!(vf.block_for_hq_unit(Some(10)));
        assert!(vf.block_for_hq_unit(Some(11)));
        assert!(vf.block_for_hq_unit(Some(12)));
        assert!(vf.block_for_hq_unit(Some(13)));
        assert!(vf.block_for_hq_unit(Some(14)));
        assert!(vf.block_for_hq_unit(Some(15)));
        assert!(vf.block_for_hq_unit(Some(16)));
        assert!(vf.block_for_hq_unit(Some(17)));
        assert!(vf.block_for_hq_unit(Some(18)));
        assert!(vf.block_for_hq_unit(Some(19)));
        assert!(!vf.block_for_hq_unit(None));
        assert!(!vf.block_for_hq_unit(Some(20)));

        // but perfect active of consonant stems is ok
        let vf = HcGreekVerbForm {
            verb: cons_stem_verb,
            person: Some(HcPerson::Third),
            number: Some(HcNumber::Plural),
            tense: HcTense::Perfect,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        assert!(vf.block_for_hq_unit(Some(2)));
        assert!(!vf.block_for_hq_unit(Some(3)));
    }

    #[test]
    fn block_middle_passive() {
        let luw = "λω, λσω, ἔλῡσα, λέλυκα, λέλυμαι, ἐλύθην";
        let luwverb = Arc::new(HcGreekVerb::from_string(1, luw, REGULAR, 0).unwrap());

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Aorist,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Aorist,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // no change of voice: not blocked
        assert!(!a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // middle to passive, both present tense: blocked
        assert!(a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // passive to middle, both present tense: blocked
        assert!(a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Aorist,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // passive to middle, first one is aorist: not blocked
        assert!(!a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Aorist,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // passive to middle, second one is aorist: not blocked
        assert!(!a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // passive to middle, first one is future: not blocked
        assert!(!a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // passive to middle, second one is future: not blocked
        assert!(!a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Future,
            voice: HcVoice::Middle,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // passive to middle, both future: not blocked
        assert!(!a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // passive to active, both present: not blocked
        assert!(!a.block_middle_passive(&b));

        let a = HcGreekVerbForm {
            verb: luwverb.clone(),
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Active,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        let b = HcGreekVerbForm {
            verb: luwverb,
            person: Some(HcPerson::First),
            number: Some(HcNumber::Singular),
            tense: HcTense::Present,
            voice: HcVoice::Passive,
            mood: HcMood::Indicative,
            gender: None,
            case: None,
        };
        // active to passive, both present: not blocked
        assert!(!a.block_middle_passive(&b));
    }
}
