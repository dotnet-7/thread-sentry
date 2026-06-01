use thread_sentry::{Mutex, RwLock, init, report_issues};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct BankAccount {
    id: u64,
    balance: f64,
}

struct Bank {
    accounts: Vec<Arc<Mutex<BankAccount>>>,
}

impl Bank {
    fn new(num_accounts: usize) -> Self {
        let accounts = (0..num_accounts)
            .map(|i| {
                Arc::new(Mutex::new(BankAccount {
                    id: i as u64,
                    balance: 1000.0,
                }))
            })
            .collect();
        Self { accounts }
    }

    fn transfer(&self, from_idx: usize, to_idx: usize, amount: f64) -> Result<(), String> {
        if from_idx >= self.accounts.len() || to_idx >= self.accounts.len() {
            return Err("Invalid account index".to_string());
        }

        let from = Arc::clone(&self.accounts[from_idx]);
        let to = Arc::clone(&self.accounts[to_idx]);

        let mut from_account = from.lock();
        let mut to_account = to.lock();

        if from_account.balance < amount {
            return Err("Insufficient funds".to_string());
        }

        from_account.balance -= amount;
        to_account.balance += amount;

        Ok(())
    }

    fn get_total_balance(&self) -> f64 {
        self.accounts
            .iter()
            .map(|acc| acc.lock().balance)
            .sum()
    }
}

fn main() {
    init();
    
    println!("=========================================");
    println!("Thread-Sentry Real-World Example");
    println!("Banking System Simulation");
    println!("=========================================\n");

    let bank = Arc::new(Bank::new(10));
    let initial_balance = bank.get_total_balance();
    println!("Initial total balance: ${:.2}\n", initial_balance);

    let start = Instant::now();
    let num_transactions = 1000;
    let num_threads = 8;

    let mut handles = vec![];
    for thread_id in 0..num_threads {
        let bank_clone = Arc::clone(&bank);
        let h = thread::spawn(move || {
            let tx_per_thread = num_transactions / num_threads;
            for i in 0..tx_per_thread {
                let from = (thread_id * 10 + i) % 10;
                let to = (from + 1) % 10;
                
                match bank_clone.transfer(from, to, 10.0) {
                    Ok(_) => {}
                    Err(e) => {
                        if i % 100 == 0 {
                            println!("Thread {} transaction failed: {}", thread_id, e);
                        }
                    }
                }
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    let duration = start.elapsed();
    let final_balance = bank.get_total_balance();

    println!("Transactions completed in {:?}", duration);
    println!("Final total balance: ${:.2}", final_balance);
    println!("Balance preserved: {}", 
        if (final_balance - initial_balance).abs() < 0.01 {
            "✓ YES"
        } else {
            "✗ NO (Race condition detected!)"
        }
    );

    println!("\n");
    report_issues();

    println!("\n=========================================");
    println!("Example: Concurrent Cache with RwLock");
    println!("=========================================\n");

    let cache: Arc<RwLock<Vec<(String, u64)>>> = Arc::new(RwLock::new(Vec::new()));
    let mut handles = vec![];

    for i in 0..4 {
        let cache_clone = Arc::clone(&cache);
        let h = thread::spawn(move || {
            for j in 0..100 {
                if j % 2 == 0 {
                    let mut writer = cache_clone.write();
                    writer.push((format!("key-{}-{}", i, j), i * 100 + j));
                } else {
                    let reader = cache_clone.read();
                    let _ = reader.len();
                }
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Cache size: {} entries", cache.read().len());
    println!("\n");
    report_issues();
}