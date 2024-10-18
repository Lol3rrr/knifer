use super::*;

pub struct PerRoundAnalysis {}

impl PerRoundAnalysis {
    pub fn new() -> Self {
        Self {}
    }
}

impl Analysis for PerRoundAnalysis {
    #[tracing::instrument(name = "PerRoundAnalysis", skip(self, input))]
    fn analyse(
        &self,
        input: AnalysisInput,
    ) -> Result<
        Box<
            dyn FnOnce(
                    &mut diesel_async::pg::AsyncPgConnection,
                ) -> core::pin::Pin<
                    Box<
                        (dyn core::future::Future<Output = Result<(), diesel::result::Error>>
                             + Send
                             + '_),
                    >,
                > + Send,
        >,
        (),
    > {
        let result = analysis::perround::parse(input.data()).unwrap();

        let values: Vec<crate::models::DemoRound> = result
            .rounds
            .into_iter()
            .enumerate()
            .map(|(i, r)| crate::models::DemoRound {
                demo_id: input.demoid.clone(),
                round_number: i as i16,
                start_tick: r.start as i64,
                end_tick: r.end as i64,
                win_reason: serde_json::to_string(&r.winreason).unwrap(),
                events: serde_json::to_value(&r.events).unwrap(),
            })
            .collect();

        Ok(Box::new(move |connection| {
            Box::pin(async move {
                let query = diesel::dsl::insert_into(crate::schema::demo_round::dsl::demo_round)
                    .values(&values)
                    .on_conflict_do_nothing();

                query.execute(connection).await?;

                Ok(())
            })
        }))
    }
}
