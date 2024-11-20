use crate::driver::PushPrql;

pub fn add<LHS, RHS>(lhs: LHS, rhs: RHS) -> Add<LHS, RHS> {
    Add { lhs, rhs }
}

pub fn avg<Expr>(expr: Expr) -> Avg<Expr> {
    Avg { expr }
}

pub fn case<const N: usize, Cond, Then>(
    cases: impl Into<[WhenThen<Cond, Then>; N]>,
) -> Case<N, Cond, Then> {
    Case {
        cases: cases.into(),
    }
}

pub fn count() -> Count {
    Count {}
}

pub fn sub<LHS, RHS>(lhs: LHS, rhs: RHS) -> Sub<LHS, RHS> {
    Sub { lhs, rhs }
}

pub fn sum<Expr>(expr: Expr) -> Sum<Expr> {
    Sum { expr }
}

pub fn when<Cond>(cond: Cond) -> When<Cond> {
    When { cond }
}

pub struct Empty;

impl PushPrql for Empty {
    fn push_to_driver(&self, _driver: &mut crate::driver::Driver) {}
}

pub struct Add<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> PushPrql for Add<LHS, RHS>
where
    LHS: PushPrql,
    RHS: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.lhs.push_to_driver(driver);
        driver.push(" + ");
        self.rhs.push_to_driver(driver);
    }
}

pub struct Avg<Expr> {
    pub expr: Expr,
}

impl<Expr> PushPrql for Avg<Expr>
where
    Expr: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        driver.push("average ");
        self.expr.push_to_driver(driver);
    }
}

pub struct Case<const N: usize, Cond, Then> {
    pub cases: [WhenThen<Cond, Then>; N],
}

impl<const N: usize, Cond, Then> Case<N, Cond, Then> {
    pub fn otherwise<Otherwise>(
        self,
        otherwise: Otherwise,
    ) -> CaseOtherwise<N, Cond, Then, Otherwise> {
        CaseOtherwise {
            cases: self.cases,
            otherwise,
        }
    }
}

impl<const N: usize, Cond, Then> PushPrql for Case<N, Cond, Then>
where
    Cond: PushPrql,
    Then: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        driver.push("case [");
        for (i, case) in self.cases.iter().enumerate() {
            if i > 0 {
                driver.push(',');
            }
            driver.push(' ');
            case.push_to_driver(driver);
        }
        driver.push(" ]");
    }
}

pub struct CaseOtherwise<const N: usize, Cond, Then, Otherwise> {
    pub cases: [WhenThen<Cond, Then>; N],
    pub otherwise: Otherwise,
}

impl<const N: usize, Cond, Then, Otherwise> PushPrql for CaseOtherwise<N, Cond, Then, Otherwise>
where
    Cond: PushPrql,
    Then: PushPrql,
    Otherwise: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        driver.push("case [");
        for (i, case) in self.cases.iter().enumerate() {
            if i > 0 {
                driver.push(',');
            }
            driver.push(' ');
            case.push_to_driver(driver);
            driver.push(',');
        }
        driver.push(" true => ");
        self.otherwise.push_to_driver(driver);
        driver.push(" ]");
    }
}

pub struct Count {}

impl PushPrql for Count {
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        driver.push("count []");
    }
}

pub struct Sub<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> PushPrql for Sub<LHS, RHS>
where
    LHS: PushPrql,
    RHS: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.lhs.push_to_driver(driver);
        driver.push(" - ");
        self.rhs.push_to_driver(driver);
    }
}

pub struct Sum<Expr> {
    pub expr: Expr,
}

impl<Expr> PushPrql for Sum<Expr>
where
    Expr: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        driver.push("sum ");
        self.expr.push_to_driver(driver);
    }
}

pub struct When<Cond> {
    pub cond: Cond,
}

impl<Cond> When<Cond> {
    pub fn then<Then>(self, then: Then) -> WhenThen<Cond, Then> {
        WhenThen {
            cond: self.cond,
            then,
        }
    }
}

pub struct WhenThen<Cond, Then> {
    pub cond: Cond,
    pub then: Then,
}

impl<Cond, Then> PushPrql for WhenThen<Cond, Then>
where
    Cond: PushPrql,
    Then: PushPrql,
{
    fn push_to_driver(&self, driver: &mut crate::driver::Driver) {
        self.cond.push_to_driver(driver);
        driver.push(" => ");
        self.then.push_to_driver(driver);
    }
}
