//! 木の増加と減少の様子をシミュレートする

use hakoniwa::{Context, EventContents, GeneratedData, ObjectType, Point, Time};
use num_bigint::BigUint;
use num_traits::identities::Zero;
use rand::Rng;

/// 木の種ごとの情報
#[derive(Debug, Clone)]
struct TreeTypeInfo {
  /// 寿命の年数の平均
  lifetime: usize,
  /// 寿命の偏差
  life_mu: f64,
  /// 周囲に必要なスペース
  /// 生きた年数によって変わってくる
  required_space: fn(BigUint) -> BigUint,
  /// 子孫を生産し始める年
  can_generate_children_year: usize,
  /// 生産する子孫の数を一日ごとに判定する
  generate_children_quantity: fn(BigUint) -> usize,
  /// 子供を散らばらせる範囲
  children_range_x: usize,
  children_range_y: usize,
}

/// 個別の木の情報
#[derive(Debug, Clone)]
struct TreeInfo {
  treetype: TreeType,
  /// 寿命（年）
  lifetime: Time,
  /// 周囲に必要なスペース
  /// 生きた年数によって変わってくる
  required_space: fn(BigUint) -> BigUint,
  /// 生産する子孫の数を一日ごとに判定する
  generate_children_quantity: fn(BigUint) -> usize,
  /// 子供を散らばらせる範囲
  children_range_x: usize,
  children_range_y: usize,
  /// たどり着いた位置
  generated_point: Point,
}

impl ObjectType for TreeInfo {
  fn name(&self) -> String {
    match self.treetype {
      TreeType::Pine => "松".to_string(),
      TreeType::Sakura => "桜".to_string(),
      TreeType::Ginkgo => "いちょう".to_string(),
      TreeType::Cedar => "杉".to_string(),
    }
  }
  fn generated_point(&self) -> Point {
    self.generated_point.clone()
  }
}

/// 木の種類
#[derive(Debug, Clone)]
enum TreeType {
  /// 松
  Pine,
  /// 櫻
  Sakura,
  /// 銀杏
  Ginkgo,
  /// 杉
  Cedar,
}

const DAY_OF_TIME: usize = 24;
const YEAR_OF_DAY: usize = 365;

impl TreeType {
  fn info(&self) -> TreeTypeInfo {
    match self {
      TreeType::Pine => TreeTypeInfo {
        lifetime: 60,
        life_mu: 15.0,
        required_space: |year| {
          if year < BigUint::from(15 as usize) {
            BigUint::from(10 as usize)
          } else if year < BigUint::from(30 as usize) {
            BigUint::from(20 as usize)
          } else {
            BigUint::from(30 as usize)
          }
        },
        can_generate_children_year: 25,
        generate_children_quantity: |day| {
          let mut rng = rand::thread_rng();
          if day < BigUint::from(YEAR_OF_DAY / 4 * 3) {
            0
          } else if day < BigUint::from(YEAR_OF_DAY / 8 * 7) {
            (rng.gen_range(0.0..1.0) * 30.0) as usize
          } else {
            0
          }
        },
        children_range_x: 75,
        children_range_y: 75,
      },
      TreeType::Sakura => TreeTypeInfo {
        lifetime: 50,
        life_mu: 5.0,
        required_space: |year| {
          if year < BigUint::from(20 as usize) {
            BigUint::from(20 as usize)
          } else {
            BigUint::from(30 as usize)
          }
        },
        can_generate_children_year: 30,
        generate_children_quantity: |day| {
          let mut rng = rand::thread_rng();
          if day < BigUint::from(YEAR_OF_DAY / 4 * 3) {
            0
          } else if day < BigUint::from(YEAR_OF_DAY / 8 * 7) {
            (rng.gen_range(0.0..1.0) * 10.0) as usize
          } else {
            0
          }
        },
        children_range_x: 30,
        children_range_y: 30,
      },
      TreeType::Ginkgo => TreeTypeInfo {
        lifetime: 70,
        life_mu: 20.0,
        required_space: |year| {
          if year < BigUint::from(20 as usize) {
            BigUint::from(10 as usize)
          } else if year < BigUint::from(40 as usize) {
            BigUint::from(30 as usize)
          } else {
            BigUint::from(40 as usize)
          }
        },
        can_generate_children_year: 20,
        generate_children_quantity: |day| {
          let mut rng = rand::thread_rng();
          if day < BigUint::from(YEAR_OF_DAY / 4 * 3) {
            0
          } else if day < BigUint::from(YEAR_OF_DAY / 8 * 7) {
            (rng.gen_range(0.0..1.0) * 30.0) as usize
          } else {
            0
          }
        },
        children_range_x: 100,
        children_range_y: 100,
      },
      TreeType::Cedar => TreeTypeInfo {
        lifetime: 65,
        life_mu: 25.0,
        required_space: |year| {
          if year < BigUint::from(10 as usize) {
            BigUint::from(10 as usize)
          } else if year < BigUint::from(40 as usize) {
            BigUint::from(25 as usize)
          } else {
            BigUint::from(30 as usize)
          }
        },
        can_generate_children_year: 20,
        generate_children_quantity: |day| {
          let mut rng = rand::thread_rng();
          if day < BigUint::from(YEAR_OF_DAY / 4 * 3) {
            0
          } else if day < BigUint::from(YEAR_OF_DAY / 8 * 7) {
            (rng.gen_range(0.0..1.0) * 45.0) as usize
          } else {
            0
          }
        },
        children_range_x: 80,
        children_range_y: 80,
      },
    }
  }
}

#[derive(Debug, Clone)]
enum TreeEvent {
  GenerateChildren { parent: String, point: Point },
  DeadTree { id: String },
}

impl EventContents for TreeEvent {
  fn move_object_opt(&self) -> Option<(String, Point)> {
    None
  }
  fn target_object_opt(&self) -> Option<String> {
    None
  }
  fn lifetime(&self) -> Option<Time> {
    Some(Time::new(
      BigUint::from(DAY_OF_TIME),
      BigUint::from(DAY_OF_TIME),
      BigUint::from(YEAR_OF_DAY),
    ))
  }
  fn do_object(&self) -> String {
    match self {
      TreeEvent::GenerateChildren { parent, .. } => parent.to_string(),
      TreeEvent::DeadTree { id } => id.to_string(),
    }
  }
}

fn generate_children(ctx: &Context<TreeEvent, TreeInfo>) -> GeneratedData<TreeEvent, TreeInfo> {
  let now = &ctx.time;
  todo!()
}

fn main() {
  let zero = Time::new(
    BigUint::zero(),
    BigUint::from(DAY_OF_TIME),
    BigUint::from(YEAR_OF_DAY),
  );
  let object_lst = vec![];
  let mut context: Context<TreeEvent, TreeInfo> = Context::new(zero, object_lst);
}
