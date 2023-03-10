//! これは「世界」をシミュレートするために必要なものを定義したライブラリである。
//! 「自分が操作しないで自律的に良い感じに複雑なことが起こっている様子を観察したい」という欲求を実現するために作成した。
//!
//! 導入した概念として
//! - 時間
//! - 座標場としての位置
//! - オブジェクト
//!   - 人も機械も植物も全てオブジェクトである
//!   - `ObjectType`トレイトを実装している型を元にオブジェクトは自動生成される
//! - イベント
//!   - 他のオブジェクトに対して干渉するための設けられている
//!   - `EventContents`トレイトを実装している型から児童で生成される
//! - 忘却
//!   - 情報は忘れられるものである
//!   - イベントには寿命が設定され、それを迎えると自動で削除される
//! - context
//!   - 世界の情報が全て詰まっている
//!   - 実装者はcontextから以下のものを生成する関数を用意する必要がある
//!     - イベント
//!     - 生成されるオブジェクト
//!     - 消滅するオブジェクトのID
//!
//! がある

use num_bigint::BigUint;
use num_traits::identities::One;
use rustc_hash::FxHashMap;
use std::time::SystemTime;

/// 時間に関するデータ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Time {
  /// 単位時間がどれくらいたったのかを計算する
  all: BigUint,
  /// 一日にかかる単位時間
  one_day_of_time: BigUint,
  /// 何日目か
  day: BigUint,
  /// 一日に満たない余りの単位時間数
  remainder_time: BigUint,
  /// 一年にかかる日数
  one_year_of_day: BigUint,
  /// 何年目か
  year: BigUint,
  /// 一年に満たない余りの日数
  remainder_day: BigUint,
}

impl Time {
  /// 時間の新たな生成
  pub fn new(all: BigUint, one_day_of_time: BigUint, one_year_of_day: BigUint) -> Self {
    let day = &all / &one_day_of_time;
    let remainder_time = &all % &one_day_of_time;
    let year = &day / &one_year_of_day;
    let remainder_day = &all % &one_year_of_day;
    Time {
      all,
      one_day_of_time,
      day,
      remainder_time,
      one_year_of_day,
      year,
      remainder_day,
    }
  }

  /// 時間を任意の量進める
  pub fn plus(&mut self, time: BigUint) {
    let all = &self.all + &time;
    let new_remainder_time = &self.remainder_time + &time;
    let plus_day = &new_remainder_time / &self.one_day_of_time;
    let day = &self.day + &plus_day;
    let remainder_time = &new_remainder_time % &self.one_day_of_time;
    let new_remainder_day = &self.remainder_day + &plus_day;
    let year = &self.year + (&new_remainder_day / &self.one_year_of_day);
    let remainder_day = &new_remainder_day % &self.one_year_of_day;
    *self = Time {
      all,
      day,
      remainder_time,
      year,
      remainder_day,
      ..self.clone()
    }
  }

  /// 時間を一単位時間進める
  pub fn plus_one(&mut self) {
    self.plus(BigUint::one())
  }

  /// 年や日数にかかる単位時間を変化させられる
  pub fn change_rule(&mut self, one_day_of_time: BigUint, one_year_of_day: BigUint) {
    let plus_day = &self.remainder_time / &one_day_of_time;
    let day = &self.day + &plus_day;
    let remainder_time = &self.remainder_time % &one_day_of_time;
    let new_remainder_day = &self.remainder_day + &plus_day;
    let year = &self.year + (&new_remainder_day / &one_year_of_day);
    let remainder_day = new_remainder_day % &one_year_of_day;
    *self = Time {
      one_day_of_time,
      day,
      remainder_time,
      one_year_of_day,
      year,
      remainder_day,
      ..self.clone()
    }
  }
}

/// 地図上での「地点」を表す。
/// どの座標系を採用しているかは実装者に任せるが、一応右手系を想定している
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Point {
  x: BigUint,
  y: BigUint,
}

/// オブジェクトの種類やオブジェクトそのものの情報
pub trait ObjectType: Clone {
  /// オブジェクトの種類の名前
  fn name(&self) -> String;
  /// そのオブジェクトが生み出された場所
  fn generated_point(&self) -> Point;
}

/// 世界に存在する「モノ」
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object<T: ObjectType + ?Sized> {
  /// 生成時刻
  pub generated_time: Time,
  /// 現在地
  pub point: Point,
  /// オブジェクトの種類
  pub object_type: T,
}

/// イベントを生成するために必要な情報
pub trait EventContents: Clone {
  // /// イベントの発生により生成されるオブジェクトがある場合はそのオブジェクトを返す
  fn generate_object_opt(&self) -> Option<String>;
  /// イベントの発生により削除されるオブジェクトがある場合はそのID
  fn remove_object_opt(&self) -> Option<String>;
  /// オブジェクトを移動させる場合に発生する
  /// 対象のオブジェクトのIDと移動先の地点
  fn move_object_opt(&self) -> Option<(String, Point)>;
  /// eventの寿命
  /// Noneの場合は永久
  fn lifetime(&self) -> Option<Time>;
  /// イベントを発生させた主体のオブジェクトのID
  fn do_object(&self) -> String;
  /// オブジェクト間に起こるイベントの場合に、そのイベントの対象となったオブジェクトのID
  fn target_object_opt(&self) -> Option<String>;
}

/// 起きるイベント
#[derive(Debug, Clone)]
pub struct Event<T: EventContents> {
  /// イベントが起きた時刻
  pub generated_time: Time,
  /// イベントの寿命
  pub lifetime: Option<Time>,
  /// イベントの中身
  pub contents: T,
  /// イベントを発生させた主体のオブジェクトのID
  pub do_object: String,
  /// オブジェクト間に起こるイベントの場合に、そのイベントの対象となったオブジェクトのID
  pub target_object: Option<String>,
}

/// 世界の状態を保持しているもの
#[derive(Debug, Clone)]
pub struct Context<T: EventContents, U: ObjectType> {
  /// 現在の時刻
  pub time: Time,
  /// 記憶されているイベント
  pub memory: Vec<Event<T>>,
  /// 現在存在する全てのオブジェクト
  pub objects: FxHashMap<String, Object<U>>,
}

/// 世界の状態に応じて変化する情報
#[derive(Debug, Clone)]
pub struct GeneratedData<T: EventContents, U: ObjectType> {
  /// 新たに起きたイベント
  pub events: Vec<T>,
  /// 新たに生成されたオブジェクト
  pub generate_objects: Vec<U>,
  /// 新たに消滅したオブジェクト
  pub remove_objects: Vec<String>,
}

/// 新たな情報を生成するための関数
pub type Generater<T, U> = fn(&Context<T, U>) -> GeneratedData<T, U>;

/// 単位時間を一つだけ進め、その結果起こるイベントをすべて記録し、世界を更新する
/// - `T`は「イベントの具体的な中身」
/// - `U`は「オブジェクトの具体的な中身」
pub fn run<T: EventContents, U: ObjectType>(
  ctx: &mut Context<T, U>,
  generate_functions: Vec<Generater<T, U>>,
) -> Vec<GeneratedData<T, U>> {
  ctx.time.plus_one();
  let now = ctx.time.clone();
  let new_memory = ctx
    .memory
    .iter()
    .filter(|e| {
      if let Some(lifetime) = &e.lifetime {
        &e.generated_time.all + &lifetime.all < now.all
      } else {
        // Noneの場合は永久に残るものなので残す
        true
      }
    })
    .cloned()
    .collect::<Vec<_>>();
  ctx.memory = new_memory;
  let mut new_events = Vec::new();
  let mut new_objects = Vec::new();
  let mut remove_object_id = Vec::new();
  let mut generated_data_lst = Vec::new();
  for f in generate_functions.iter() {
    let generated_data = f(ctx);
    generated_data_lst.push(generated_data.clone());
    let e_lst = generated_data.events;
    for e in e_lst.iter() {
      let event = Event {
        generated_time: now.clone(),
        lifetime: e.lifetime(),
        contents: e.clone(),
        do_object: e.do_object(),
        target_object: e.target_object_opt(),
      };
      new_events.push(event);
    }
    let mut r = generated_data.remove_objects;
    remove_object_id.append(&mut r);
    let o_lst = generated_data.generate_objects;
    for o in o_lst.iter() {
      let object = Object {
        generated_time: now.clone(),
        point: o.generated_point(),
        object_type: o.clone(),
      };
      let id = generate_object_id(&o.name(), &o.generated_point(), &now.all);
      new_objects.push((id, object));
    }
  }
  for object_id in remove_object_id.iter() {
    ctx.objects.remove(object_id);
  }
  ctx.memory.append(&mut new_events);
  for e in new_events.iter() {
    if let Some((id, point)) = e.contents.move_object_opt() {
      if let Some(obj) = ctx.objects.get(&id) {
        let new_obj = Object {
          point,
          ..obj.clone()
        };
        ctx.objects.insert(id, new_obj);
      }
    }
  }
  for object_id in remove_object_id.iter() {
    ctx.objects.remove(object_id);
  }
  for (object_id, object) in new_objects.iter() {
    ctx.objects.insert(object_id.clone(), object.clone());
  }
  generated_data_lst
}

/// オブジェクトのIDを自動で生成する
/// <object_type><生成された地点><生成された単位時間><実世界の生成されたときの時刻>
/// で文字列生成してさらにBase64エンコード
fn generate_object_id(object_name: &str, point: &Point, generate_time: &BigUint) -> String {
  let now = SystemTime::now();
  let str = format!("{object_name}{point:?}{generate_time:?}{now:?}");
  base64::encode(str.as_bytes())
}
