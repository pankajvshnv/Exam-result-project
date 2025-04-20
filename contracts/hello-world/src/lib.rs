#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// Define the result status enum to track verification status
#[contracttype]
#[derive(Clone)]
pub struct ResultStats {
    pub verified: u64,   // Count of verified exam results
    pub pending: u64,    // Count of results pending verification
    pub total: u64       // Total number of results in the system
}

// Constant for accessing global stats
const ALL_RESULTS: Symbol = symbol_short!("ALL_RES");

// Structure to store exam result information
#[contracttype]
#[derive(Clone)]
pub struct ExamResult {
    pub result_id: u64,
    pub student_id: String,
    pub exam_name: String,
    pub score: u64,
    pub timestamp: u64,
    pub is_verified: bool
}

// For mapping result_id to exam results
#[contracttype]
pub enum ResultBook {
    Result(u64)
}

// For generating unique result IDs
const RESULT_COUNT: Symbol = symbol_short!("R_COUNT");

#[contract]
pub struct ExamResultContract;

#[contractimpl]
impl ExamResultContract {
    // Submit a new exam result to the blockchain
    pub fn submit_result(env: Env, student_id: String, exam_name: String, score: u64) -> u64 {
        // Generate a unique result ID
        let mut result_count: u64 = env.storage().instance().get(&RESULT_COUNT).unwrap_or(0);
        result_count += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create the exam result entry
        let exam_result = ExamResult {
            result_id: result_count,
            student_id: student_id,
            exam_name: exam_name,
            score: score,
            timestamp: timestamp,
            is_verified: false
        };
        
        // Update statistics
        let mut stats = Self::get_statistics(env.clone());
        stats.pending += 1;
        stats.total += 1;
        
        // Store the exam result and updated stats
        env.storage().instance().set(&ResultBook::Result(result_count), &exam_result);
        env.storage().instance().set(&ALL_RESULTS, &stats);
        env.storage().instance().set(&RESULT_COUNT, &result_count);
        
        // Set storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Exam result submitted with ID: {}", result_count);
        
        return result_count;
    }
    
    // Verify an exam result (only callable by authorized verifiers)
    pub fn verify_result(env: Env, result_id: u64) {
        // Get the exam result
        let key = ResultBook::Result(result_id);
        let mut exam_result = env.storage().instance().get(&key).unwrap_or_else(|| {
            log!(&env, "Result with ID {} not found", result_id);
            panic!("Result not found");
        });
        
        // Ensure result is not already verified
        if exam_result.is_verified {
            log!(&env, "Result already verified");
            panic!("Result already verified");
        }
        
        // Update verification status
        exam_result.is_verified = true;
        
        // Update statistics
        let mut stats = Self::get_statistics(env.clone());
        stats.verified += 1;
        stats.pending -= 1;
        
        // Store updated data
        env.storage().instance().set(&key, &exam_result);
        env.storage().instance().set(&ALL_RESULTS, &stats);
        
        // Set storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Result ID: {} has been verified", result_id);
    }
    
    // Retrieve an exam result by ID
    pub fn get_result(env: Env, result_id: u64) -> ExamResult {
        let key = ResultBook::Result(result_id);
        
        env.storage().instance().get(&key).unwrap_or_else(|| {
            log!(&env, "Result with ID {} not found", result_id);
            panic!("Result not found");
        })
    }
    
    // Get statistics about results in the system
    pub fn get_statistics(env: Env) -> ResultStats {
        env.storage().instance().get(&ALL_RESULTS).unwrap_or(ResultStats {
            verified: 0,
            pending: 0,
            total: 0
        })
    }
}