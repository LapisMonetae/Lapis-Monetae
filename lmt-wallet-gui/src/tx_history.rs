use regex::Regex;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub tx_id: String,
    pub direction: TxDirection,
    pub amount: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TxDirection {
    Incoming,
    Outgoing,
}

impl std::fmt::Display for TxDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TxDirection::Incoming => write!(f, "IN"),
            TxDirection::Outgoing => write!(f, "OUT"),
        }
    }
}

pub fn parse_transactions(output: &str) -> Vec<Transaction> {
    let tx_re = Regex::new(r"\b[a-fA-F0-9]{64}\b").unwrap();
    let amount_re = Regex::new(r"([+-]?\d+(?:\.\d+)?)\s+(?:LMT|TLMT|SLMT|DLMT)\b").unwrap();
    let mut txs = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        // Skip lines starting with "exit code"
        if line.trim_start().to_lowercase().starts_with("exit code") {
            continue;
        }

        if let Some(cap) = tx_re.find(line) {
            let tx_id = cap.as_str().to_string();
            let context = lines[i.saturating_sub(1)..=(i + 2).min(lines.len() - 1)].join(" ");
            let context_lower = context.to_lowercase();

            let direction = if context_lower.contains("received") || context_lower.contains("inbound") {
                TxDirection::Incoming
            } else {
                TxDirection::Outgoing
            };

            let amount = amount_re
                .captures(&context)
                .map(|c| {
                    let num = &c[1];
                    // Find which currency matched
                    let rest = &context[c.get(0).unwrap().start()..];
                    let currency_re = Regex::new(r"[+-]?\d+(?:\.\d+)?\s+(LMT|TLMT|SLMT|DLMT)").unwrap();
                    let cur = currency_re.captures(rest)
                        .map(|cc| cc[1].to_string())
                        .unwrap_or_else(|| "LMT".into());
                    format!("{} {}", num, cur)
                })
                .unwrap_or_else(|| "\u{2014}".into());

            let status = if context_lower.contains("pending") {
                "pending".into()
            } else {
                "confirmed".into()
            };

            txs.push(Transaction {
                tx_id,
                direction,
                amount,
                status,
            });
        }
    }
    txs.truncate(30);
    txs
}
