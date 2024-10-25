use serde::{Deserialize, Serialize};
use std::{fs::{File, OpenOptions}, io::{self, BufReader, BufWriter, Write}};

const FILE_NAME: &str = "tasks.json";

fn main() {
	let mut task_list = TaskList::load();

	loop {
		let mut buffer = String::new();
		io::stdin().read_line(&mut buffer).unwrap();

		let action = Action::build(buffer.split(' ')).unwrap_or_else(|err| {
			eprintln!("Invalid arg: {err}");
			Action::PASS
		});
	
		action.execute(&mut task_list);
	}
}

#[derive(Deserialize, Serialize, Debug)]
struct TaskList {
	tasks: Vec<Task>,
}

impl TaskList {
	pub fn create() -> TaskList {
		TaskList {
			tasks: vec![]
		}
	}

	pub fn load() -> TaskList {
		let file = File::open(FILE_NAME)
				.expect("{FILE_NAME} should be present in runtime root.");
		let reader = BufReader::new(file);

		let task_list = serde_json::from_reader(reader).unwrap_or_else(|err| {
			eprintln!("failed to read file: {err}");
			TaskList::create()
		});

		task_list
	}

	pub fn add(&mut self, task: Task) {
		self.tasks.push(task);
		self.save();
	}

	pub fn remove(&mut self, task_id: usize) {
		self.tasks.remove(task_id);
		self.save();
	}

	fn save(&self) {
		let file = OpenOptions::new()
				.read(true)
				.write(true)
				.truncate(true)
				.open(FILE_NAME)
				.expect("{FILE_NAME} should be present in runtime root.");

		let mut writer = BufWriter::new(file);

		serde_json::to_writer(&mut writer, &self)
			.unwrap_or_else(|err| {
				eprintln!("problem saving tasks: {err}");
			});
			
		writer.flush().unwrap_or_else(|e| eprintln!("failed to flush stream: {e}"));
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
	description: String,
}

enum Action {
	ADD(String),
	REMOVE(usize),
	LIST,
	PASS
}

impl Action {
	pub fn build<'a>(
		mut args: impl Iterator<Item = &'a str>,
	) -> Result<Action, String> {
		let action = match args.next() {
			Some(arg) => arg,
			None => return Err(String::from("could not get action")),
		};

		match action.to_lowercase().as_str().trim() {
			"add" => {
				let mut description = String::new();
				for part in args {
					description.push_str(part.trim());
					description.push_str(" ");
				}
				let _ = description.split_off(description.len()-1);
				
				Ok(Action::ADD(description))
			},
			"remove" => {
				let arg = args.next().unwrap_or_else(|| {
					eprintln!("Could not parse!");
					return "";
				}).trim();

				let task_id: Result<i32, _> = arg.parse();
				if let Ok(task_id) = task_id {
					return Ok(Action::REMOVE(task_id.try_into().expect("should be able to convert i32 to usize.")))
				} else {
					return Err(format!("failed to convert {} to i32!", arg));
				}
			},
			"list" => Ok(Action::LIST),

			action => Err(format!("{} is not a defined action.", action.trim()))
		}
	}
	
	pub fn execute(&self, task_list: &mut TaskList) {
		match self {
			Action::ADD(desc) => {
				task_list.add(Task {
					description: desc.to_string(),
				});
			},
			Action::REMOVE(id) => {
				task_list.remove(*id);
			},
			Action::LIST => {
				for (i, task) in task_list.tasks.iter().enumerate() {
					println!("{} | {}", i, task.description);
				}
			},

			Action::PASS => {}
		}
	}
}