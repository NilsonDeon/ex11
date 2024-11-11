use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use rand::Rng;

struct Process {
    id: usize,
    alive: bool,
    fail_event: Arc<(Mutex<bool>, Condvar)>,
    leader_event: Arc<(Mutex<bool>, Condvar)>,
    consensus: Arc<Mutex<Consensus>>,
}

struct Consensus {
    leader: usize,
}

impl Process {
    fn new(id: usize, fail_event: Arc<(Mutex<bool>, Condvar)>, leader_event: Arc<(Mutex<bool>, Condvar)>, consensus: Arc<Mutex<Consensus>>) -> Self {
        Process {
            id,
            alive: true,
            fail_event,
            leader_event,
            consensus,
        }
    }

    fn run(&mut self) {
        while self.alive {
            thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(500..1500)));
            
            // Checa falha
            let (lock, _) = &*self.fail_event;
            if *lock.lock().unwrap() {
                self.alive = false;
                println!("Processo {} detectou a falha e parou.", self.id);
            } else {
                println!("Processo {} está ativo.", self.id);
            }

            // Verifica quem é o líder
            let consensus = self.consensus.lock().unwrap();
            if self.id == consensus.leader {
                println!("Processo {} é o líder atual.", self.id);
            } else {
                let (leader_lock, cvar) = &*self.leader_event;
                let leader_set = leader_lock.lock().unwrap();
                if !*leader_set {
                    println!("Processo {} aguardando escolha do líder.", self.id);
                    cvar.notify_all();
                }
            }
        }
        println!("Processo {} foi finalizado.", self.id);
    }
}

fn detectar_falha(processos: &[Arc<Mutex<Process>>], falha_id: usize, fail_event: Arc<(Mutex<bool>, Condvar)>, leader_event: Arc<(Mutex<bool>, Condvar)>, consensus: Arc<Mutex<Consensus>>) {
    thread::sleep(Duration::from_secs(2));
    println!("\n--- Simulando falha no processo {} ---", falha_id);
    
    {
        let (lock, _) = &*fail_event;
        let mut failed = lock.lock().unwrap();
        *failed = true;
    }

    {
        let mut consensus = consensus.lock().unwrap();
        if falha_id == consensus.leader {
            consensus.leader = (falha_id + 1) % processos.len();
            let (lock, cvar) = &*leader_event;
            *lock.lock().unwrap() = true;
            cvar.notify_all();
            println!("Novo líder escolhido: Processo {}", consensus.leader);
        }
    }
}

fn main() {
    const X: usize = 5;

    let fail_event = Arc::new((Mutex::new(false), Condvar::new()));
    let leader_event = Arc::new((Mutex::new(false), Condvar::new()));
    let consensus = Arc::new(Mutex::new(Consensus { leader: 0 }));

    let processos: Vec<_> = (0..X)
        .map(|i| Arc::new(Mutex::new(Process::new(i, fail_event.clone(), leader_event.clone(), consensus.clone()))))
        .collect();

    let mut handles = Vec::new();
    for processo in processos.iter() {
        let processo_clone = Arc::clone(processo);
        let handle = thread::spawn(move || {
            let mut processo = processo_clone.lock().unwrap();
            processo.run();
        });
        handles.push(handle);
    }

    let falha_id = 0;
    let fail_event_clone = fail_event.clone();
    let leader_event_clone = leader_event.clone();
    let consensus_clone = consensus.clone();

    let fail_handle = thread::spawn(move || {
        detectar_falha(&processos, falha_id, fail_event_clone, leader_event_clone, consensus_clone);
    });

    // Espera o fim da simulação
    fail_handle.join().unwrap();
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\n--- Consenso final: ---");
    println!("Processo {} é o líder final.", consensus.lock().unwrap().leader);
}
