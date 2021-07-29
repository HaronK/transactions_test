# Dev Comments

* Unit tests are in the **./src/process.rs** file. I was trying to cover most generic cases but they definitely don't cover all possible situations.
* It's still not clear from the description what to do when client tries to dispute not the latest transaction. Logically in this situation we should dispute all transactions starting from the last and up to the requested one. I've disabled a test for this situation.
* I still have some doubts about **dispute**/**resolve**/**chargeback** operations for **withdrawal** transaction: while final values for **resolve**/**chargeback** operations look correct, in values for **dispute** we have negative **hold**.
* I was trying to reduce amount of memory used by transaction engine by using arena instead of cloning but stuck with the borrow checker (see **arena_refs** branch). Correct implementation will require bigger refactoring.
