




  pub fn backtrack_match(s: &[u8], p: &[u8]) -> bool {
        let (mut s_idx, mut p_idx) = (0, 0);
        let (mut s_record, mut p_record) = (usize::MAX, usize::MAX);
        while s_idx < s.len() || p_idx < p.len() {
            let (orig, matcher) = (s.get(s_idx), p.get(p_idx));
            match matcher.and_then(|s|Some(*s)) {
                Some(s) => {
                    match s {
                        b'?' => match orig {
                            Some(_) => {
                                p_idx += 1;
                                s_idx += 1;
                            },
                            None => return false
                        },
                        b'*' => match orig {
                            None => {
                                p_idx += 1;
                                match p.get(p_idx).and_then(|s|Some(*s)) {
                                    Some(b'*') => continue,
                                    Some(_) => return false,
                                    None => return true,
                                }
                            }
                            Some(_) => {
                                p_record = p_idx;
                                s_record = s_idx;
                                p_idx += 1;
                                match p.get(p_idx).and_then(|s|Some(*s)) {
                                    Some(b'*') => p_idx +=1,
                                    Some(_) => {},
                                    None => return true,
                                }
                            }
                        },
                        value => match orig {
                            None => return false,
                            Some(orig) => {
                                if value == *orig {
                                    p_idx += 1;
                                    s_idx += 1;
                                } else {
                                    if p_record != usize::MAX && s_record != usize::MAX {
                                        p_idx = p_record;
                                        s_idx = s_record + 1;
                                    } else {
                                        return false;
                                    }
                                }
                            }
                        },
                    }
                }
                None => {
                    if p_record != usize::MAX && s_record != usize::MAX {
                        p_idx = p_record;
                        s_idx = s_record + 1;
                    } else {
                        return false;
                    }
                }
            }
        }
    
        s_idx >= s.len() && p_idx >= p.len()
    }