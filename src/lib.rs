use num_bigint::BigUint;
use num_traits::identities::One;
use std::time::SystemTime;
use rustc_hash::FxHashMap;

/// 現在の時間に関するデータ
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Now {
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

impl Now {
  pub fn new(all: BigUint, one_day_of_time: BigUint, one_year_of_day: BigUint) -> Self {
    let day = &all / &one_day_of_time;
    let remainder_time = &all % &one_day_of_time;
    let year = &day / &one_year_of_day;
    let remainder_day = &all % &one_year_of_day;
    Now {
      all,
      one_day_of_time,
      day,
      remainder_time,
      one_year_of_day,
      year,
      remainder_day,
    }
  }

  pub fn plus(&mut self, time: BigUint) {
    let all = &self.all + &time;
    let new_remainder_time = &self.remainder_time + &time;
    let plus_day = &new_remainder_time / &self.one_day_of_time;
    let day = &self.day + &plus_day;
    let remainder_time = &new_remainder_time % &self.one_day_of_time;
    let new_remainder_day = &self.remainder_day + &plus_day;
    let year = &self.year + (&new_remainder_day / &self.one_year_of_day);
    let remainder_day = &new_remainder_day % &self.one_year_of_day;
    *self = Now {
      all,
      day,
      remainder_time,
      year,
      remainder_day,
      ..self.clone()
    }
  }

  pub fn plus_one(&mut self) {
    self.plus(BigUint::one())
  }

  pub fn change_rule(&mut self, one_day_of_time: BigUint, one_year_of_day: BigUint) {
    let plus_day = &self.remainder_time / &one_day_of_time;
    let day = &self.day + &plus_day;
    let remainder_time = &self.remainder_time % &one_day_of_time;
    let new_remainder_day = &self.remainder_day + &plus_day;
    let year = &self.year + (&new_remainder_day / &one_year_of_day);
    let remainder_day = new_remainder_day % &one_year_of_day;
    *self = Now {
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

pub trait ObjectType {
  fn name(&self) -> String;
  fn lifetime(&self) -> BigUint;
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object<T: ObjectType + ?Sized> {
  /// 生成時刻
  generate_time : Now,
  /// 現在の位置
  point: Point,
  /// オブジェクトの種類
  /// ユーザーが定義する
  object_type: Box<T>,
  /// オブジェクトの寿命
  lifetime: BigUint
}

impl<O: ToString + Clone + ObjectType + ?Sized> Object<O> {
  pub fn get_object_type(&self) -> O {
    *self.object_type.clone()
  }
  pub fn get_point(&self) -> Point {
    self.point.clone()
  }
}

/// イベントの
pub trait EventContents<OT: ObjectType + ?Sized> {

  fn name(&self) -> String;
  /// イベントの発生により生成されるオブジェクトがある場合はそのオブジェクトを返す
  fn generate_object_opt(&self) -> Option<Object<OT>>;
  /// イベントの発生により削除されるオブジェクトがある場合はそのID
  fn remove_object_opt(&self) -> Option<String>;
  /// オブジェクトを移動させる場合に発生する
  /// 対象のオブジェクトのIDと移動先の地点
  fn move_object_opt(&self) -> Option<(String, Point)>;
}

#[derive(Debug, Clone)]
pub struct Event<C: EventContents<dyn ObjectType>> {
  /// イベントが発生した時刻
  time: Now,
  /// イベントが記憶される寿命
  lifespan: BigUint,
  /// イベントの中身
  contents: C,
  /// イベントを発生させた主体のオブジェクトのID
  do_object: String,
  /// オブジェクト間に起こるイベントの場合に、そのイベントの対象となったオブジェクトのID
  target_object: Option<String>,
}

impl<C: EventContents<dyn ObjectType> + Clone> Event<C> {
  pub fn get_time(&self) -> Now {
    self.time.clone()
  }
  pub fn get_lifespan(&self) -> BigUint {
    self.lifespan.clone()
  }
  pub fn get_contents(&self) -> C {
    self.contents.clone()
  }
  pub fn get_do_object_id(&self) -> String {
    self.do_object.clone()
  }
  pub fn get_target_object_id(&self) -> Option<String> {
    self.target_object.clone()
  }
  pub fn is_exists_target_object(&self) -> bool {
    self.target_object.is_some()
  }
}

#[derive(Debug, Clone)]
pub struct Context<C: EventContents<dyn ObjectType>, O: ToString + ObjectType> {
  /// 現在の時刻
  pub time: Now,
  /// 記憶されているイベント
  pub memory: Vec<Event<C>>,
  /// 現在存在する全てのオブジェクト
  pub objects: FxHashMap<String, Object<O>>,
}

pub type GenerateEvent<C, O> = fn(&Context<C, O>) -> dyn EventContents<dyn ObjectType>;

/// 単位時間を一つだけ進め、その結果起こるイベントをすべて記録する
/// - `C`は「イベントの具体的な中身」
/// - `O`は「オブジェクトの具体的な中身」
pub fn run<C: EventContents<dyn ObjectType>, O: ToString + ObjectType, OT>(ctx: &mut Context<C, O>, generate_functions: Vec<Box<GenerateEvent<C, O>>>) -> Vec<Event<C>> {
  ctx.time.plus_one();
  let now = ctx.time.clone();
  for f in generate_functions.iter() {
    let f = **f;
    let event_contents = f(ctx);
    let event_name = event_contents.name();
  }
  todo!()
}

/// オブジェクトのIDを自動で生成する
/// <object_type><生成された地点><生成された単位時間><実世界の生成されたときの時刻>
/// で文字列生成してさらにBase64エンコード
fn generate_object_id<O: core::fmt::Debug>(
  object_type: &O,
  point: &Point,
  generate_time: &BigUint,
) -> String {
  let now = SystemTime::now();
  let str = format!("{object_type:?}{point:?}{generate_time:?}{now:?}");
  base64::encode(&str.as_bytes())
}
