use cred_store::CredStore;

use crate::CommandContext;

pub fn logout<T: CredStore>(context: &mut CommandContext<T>) {
    if context.cred_store.delete().is_err() {
        println!("No credentials found.");
        return;
    }

    println!("Logged out.");
}
