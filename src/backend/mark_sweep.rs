use std::collections::HashMap;
use crate::backend::interpreter::DataType;

// Implementation of a mark-sweep garbage collector
pub(crate) struct GC<'a> {
    envs: &'a mut Vec<HashMap<String, Option<DataType>>>,
    lists: &'a mut Vec<Vec<DataType>>,
    free_lists: &'a mut Vec<usize>,
    nameless_records: &'a mut Vec<HashMap<String, DataType>>,
    free_nameless_records: &'a mut Vec<usize>,
}

impl<'a> GC<'a> {
    pub(crate) fn new(envs: &'a mut Vec<HashMap<String, Option<DataType>>>,
                      lists: &'a mut Vec<Vec<DataType>>,
                      free_lists: &'a mut Vec<usize>,
                      nameless_records: &'a mut Vec<HashMap<String, DataType>>,
                      free_nameless_records: &'a mut Vec<usize>,) -> Self
    {
        GC {
            envs,
            lists,
            free_lists,
            nameless_records,
            free_nameless_records,
        }
    }

    pub(crate) fn collect_garbage(&mut self) {
        let (marked_lists, marked_nameless_records) = self.gc_mark();
        self.gc_sweep(marked_lists, marked_nameless_records);
    }

    fn gc_sweep(&mut self, marked_lists: Vec<bool>, marked_record: Vec<bool>) {
        for (index, alive) in marked_lists.iter().enumerate() {
            if !alive {
                // replacing list with empty list, which will be re_used later
                self.lists[index] = Vec::new();
                if !self.free_lists.contains(&index) {
                    self.free_lists.push(index);
                }
            }
        }

        for (index, alive) in marked_record.iter().enumerate() {
            if !alive {
                // replacing record with empty record, which will be re_used later
                self.nameless_records[index] = HashMap::new();
                if !self.free_nameless_records.contains(&index) {
                    self.free_nameless_records.push(index);
                }
            }
        }
    }

    fn gc_mark(&mut self) -> (Vec<bool>, Vec<bool>) {
        let mut marked_lists: Vec<bool> = vec![false; self.lists.len()];
        let mut marked_records: Vec<bool> = vec![false; self.nameless_records.len()];

        let (root_lists, root_records) = self.find_root_objects();

        for root_list_index in root_lists {
            marked_lists[root_list_index] = true;
            let list = self.lists.get(root_list_index).unwrap();
            self.mark_all_reachable_from_list(list, &mut marked_lists, &mut marked_records);
        }

        for root_record_index in root_records {
            marked_records[root_record_index] = true;
            let record = self.nameless_records.get(root_record_index).unwrap();
            self.mark_all_reachable_from_record(record, &mut marked_lists, &mut marked_records);
        }

        (marked_lists, marked_records)
    }

    fn mark_all_reachable_from_list(&self, list: &Vec<DataType>, marked_lists: &mut Vec<bool>, marked_records: &mut Vec<bool>) {
        for elem in list {
            match elem {
                DataType::List(index) => {
                    // If already marked true don't need to revisit
                    if  !marked_lists[index.clone()] {
                        marked_lists[index.clone()] = true;
                        let list = self.lists.get(index.clone()).unwrap();
                        self.mark_all_reachable_from_list(list, marked_lists, marked_records);
                    }
                },
                DataType::NamelessRecord(index) => {
                    // If already marked true don't need to revisit
                    if  !marked_records[index.clone()] {
                        marked_records[index.clone()] = true;
                        let record = self.nameless_records.get(index.clone()).unwrap();
                        self.mark_all_reachable_from_record(record, marked_lists, marked_records);
                    }
                },
                _ => {}
            }
        }
    }

    fn mark_all_reachable_from_record(&self, record: &HashMap<String, DataType>, marked_lists: &mut Vec<bool>, marked_records: &mut Vec<bool>) {
        for (_, elem) in record.into_iter() {
            match elem {
                DataType::List(index) => {
                    // If already marked true don't need to revisit
                    if  !marked_lists[index.clone()] {
                        marked_lists[index.clone()] = true;
                        let list = self.lists.get(index.clone()).unwrap();
                        self.mark_all_reachable_from_list(list, marked_lists, marked_records);
                    }
                },
                DataType::NamelessRecord(index) => {
                    // If already marked true don't need to revisit
                    if  !marked_records[index.clone()] {
                        marked_records[index.clone()] = true;
                        let record = self.nameless_records.get(index.clone()).unwrap();
                        self.mark_all_reachable_from_record(record, marked_lists, marked_records);
                    }
                },
                _ => {}
            }
        }
    }

    fn find_root_objects(&self) -> (Vec<usize>, Vec<usize>) {
        let mut root_lists: Vec<usize> = Vec::new();
        let mut root_records: Vec<usize> = Vec::new();
        for env in self.envs.iter() {
            for (_, val) in env.into_iter() {
                if let Some(data_type) = val {
                    match data_type {
                        DataType::List(index) => root_lists.push(index.clone()),
                        DataType::NamelessRecord(index) => root_records.push(index.clone()),
                        _ => {}
                    }
                }
            }
        }
        (root_lists, root_records)
    }
}