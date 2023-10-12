use std::{borrow::Borrow, collections::HashMap};

use crate::{
    floodfill::{self, floodfill},
    models::Board,
    simulation::{Action, EndState},
    utils::{self, dir_to_string},
};

#[derive(Clone)]
struct NodeState {
    board_state: Board,
}

static mut EXPLORED_POSITIONS: i64 = 0;
static mut PRUNED_POSITIONS: i64 = 0;

impl NodeState {
    const MAX_SCORE: f32 = 200000.0;

    // Heuristic values
    const FILL_V: f32 = 1.0;
    const LIFE_V: f32 = 1.0;
    const LENGTH_V: f32 = 100.0;

    pub fn generate_score_array(&self) -> Vec<f32> {
        let board = &self.board_state;
        let end_state: EndState = board.get_endstate();
        let mut scores = vec![];
        for (index, snake) in board.snakes.iter().enumerate() {
            scores.push(self.calculate_raw_score_per_snake(&snake.id, &end_state, &board))
        }
        let total_score = scores.iter().fold(0.0, |acc, x| acc + x);
        return scores
            .iter()
            .map(|x| (x / total_score) * NodeState::MAX_SCORE)
            .collect();
    }

    fn calculate_raw_score_per_snake(
        &self,
        snake_id: &str,
        end_state: &EndState,
        board: &Board,
    ) -> f32 {
        if let EndState::Winner(winner) = end_state {
            if winner == snake_id {
                return NodeState::MAX_SCORE;
            } else {
                return 0.0;
            }
        }
        match end_state {
            EndState::Winner(winner) => {
                if winner == snake_id {
                    return NodeState::MAX_SCORE;
                }
                return f32::NEG_INFINITY;
            }
            EndState::Playing => { /* CONTINUE */ }
            EndState::TIE => return 0.0,
        }
        let fill_score = floodfill(board, snake_id);
        let snake = board.get_snake(snake_id);
        let health_score = snake.health;
        let length_score = snake.body.len();
        let mut final_score = (health_score as f32) * NodeState::LIFE_V;
        final_score += (length_score as f32) * NodeState::LENGTH_V;
        final_score += (fill_score as f32) * NodeState::FILL_V;
        return final_score;
    }
}

pub struct Tree {
    snake_map: HashMap<String, usize>,
    snake_vec: Vec<String>,
    root: NodeState,
}

impl Tree {
    pub const MAX_DEPTH: usize = 10;

    pub fn get_next_snake(&self, current_snake: &str) -> &str {
        let next_index = self.snake_map[current_snake] + 1;
        return &self.snake_vec[next_index % self.snake_vec.len()];
    }

    pub fn is_last_nake(&self, current_snake: &str) -> bool {
        let cur_index = self.snake_map[current_snake];
        return cur_index + 1 == self.snake_vec.len();
    }

    pub fn new(starting_board: Board) -> Self {
        let mut snake_vec = vec![];
        let mut snake_map = HashMap::new();

        for (i, snake) in starting_board.borrow().snakes.iter().enumerate() {
            let copy_snake = snake.id.clone();
            snake_vec.push(copy_snake.clone());
            snake_map.insert(copy_snake.clone(), i);
        }

        let root_node_state = NodeState {
            board_state: starting_board,
        };

        return Self {
            snake_map,
            snake_vec,
            root: root_node_state,
        };
    }

    pub fn get_best_move(&self, target_snake_id: &str) -> (i32, i32) {
        let board_state = &self.root.board_state;
        let current_snake = target_snake_id;
        let alphas = vec![NodeState::MAX_SCORE; board_state.snakes.len()];
        let (score, best_move) = self.get_score(0, &self.root, alphas, current_snake);

        println!("board state:\n{}", board_state.to_string());

        unsafe {
            println!(
                "found best move {} with score {:?} after exploring {} moves\npruned {} positions",
                dir_to_string(best_move),
                score,
                EXPLORED_POSITIONS,
                PRUNED_POSITIONS
            );
        }
        return best_move;
    }

    fn get_score(
        &self,
        depth: usize,
        node_state: &NodeState,
        alphas: Vec<f32>,
        current_snake: &str,
    ) -> (Vec<f32>, (i32, i32)) {
        let mut best_dir = (1, 0);

        if depth == Tree::MAX_DEPTH || node_state.board_state.is_terminal() {
            return (node_state.generate_score_array(), best_dir);
        }

        let mut new_alphas = alphas.clone();
        let board_state = &node_state.board_state;
        let mut max_score = vec![];

        for dir in utils::DIRECTIONS {
            // Perform alpha pruning.
            // If we found a move better than what is above us we can stop looking.
            if max_score.len() > 0
                && max_score[self.snake_map[current_snake]] > alphas[self.snake_map[current_snake]]
            {
                unsafe {
                    PRUNED_POSITIONS += 1;
                }
                break;
            }

            unsafe {
                EXPLORED_POSITIONS += 1;
            }

            let mut board_copy = board_state.clone();
            let action = Action {
                snake_id: current_snake.to_owned(),
                dir,
            };
            board_copy.execute_action(action, self.is_last_nake(current_snake));

            let new_node = NodeState {
                board_state: board_copy.clone(),
            };
            let (new_score, _) = self.get_score(
                depth + 1,
                &new_node,
                new_alphas.clone(),
                &self.get_next_snake(current_snake),
            );

            if max_score.len() == 0
                || new_score[self.snake_map[current_snake]]
                    > max_score[self.snake_map[current_snake]]
            {
                best_dir = dir;
                max_score = new_score;
                for index in 0..new_alphas.len() {
                    if index == self.snake_map[current_snake] {
                        new_alphas[index] = max_score[self.snake_map[current_snake]]
                    } else {
                        new_alphas[index] =
                            NodeState::MAX_SCORE - max_score[self.snake_map[current_snake]]
                    }
                }
            }
        }
        return (max_score, best_dir);
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test_utils::{
        self, AVOID_DEATH_ADVANCED, AVOID_DEATH_GET_FOOD, AVOID_SELF_TRAP, GET_THE_FOOD,
    };

    #[test]
    fn test_avoid_wall() {
        let game_state = test_utils::get_board();
        let tree = Tree::new(game_state);
        let best_move = dir_to_string(tree.get_best_move("long_snake"));
        assert_ne!("up", best_move)
    }

    #[test]
    fn test_avoid_death_get_food() {
        let game_state = test_utils::get_scenario(AVOID_DEATH_GET_FOOD);
        let me = game_state.you.id;
        let tree = Tree::new(game_state.board);
        let best_move = dir_to_string(tree.get_best_move(&me));
        assert_ne!(best_move, "right")
    }

    #[test]
    fn test_avoid_self_trap() {
        let game_state = test_utils::get_scenario(AVOID_SELF_TRAP);
        let me = game_state.you.id;
        let tree = Tree::new(game_state.board);
        let best_move = dir_to_string(tree.get_best_move(&me));
        assert_ne!(best_move, "up")
    }
    #[test]
    fn test_get_easy_food() {
        let game_state = test_utils::get_scenario(GET_THE_FOOD);
        let me = game_state.you.id;
        let tree = Tree::new(game_state.board);
        let best_move = dir_to_string(tree.get_best_move(&me));
        assert_eq!(best_move, "down")
    }

    #[test]
    fn test_avoid_death_advanced() {
        let game_state = test_utils::get_scenario(AVOID_DEATH_ADVANCED);
        let me = game_state.you.id;
        let tree = Tree::new(game_state.board);
        let best_move = dir_to_string(tree.get_best_move(&me));
        assert_ne!(best_move, "right")
    }
}