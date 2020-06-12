use crate::app::services::AffinitiesService;
use devand_core::UserAffinity;
use serde_derive::{Deserialize, Serialize};
use yew::{prelude::*, Properties};

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    affinities: Option<Vec<UserAffinity>>,
}

pub enum Msg {
    AffinitiesFetchOk(Vec<UserAffinity>),
    AffinitiesFetchErr,
}

pub struct AffinitiesPage {
    props: Props,
    state: State,
    #[allow(dead_code)]
    affinities_service: AffinitiesService,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

impl Component for AffinitiesPage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = State::default();

        let callback = link.callback(|result: Result<Vec<UserAffinity>, anyhow::Error>| {
            if let Ok(affinities) = result {
                Msg::AffinitiesFetchOk(affinities)
            } else {
                Msg::AffinitiesFetchErr
            }
        });

        let mut affinities_service = AffinitiesService::new(callback);

        affinities_service.restore();

        Self {
            props,
            state,
            affinities_service,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AffinitiesFetchOk(affinities) => {
                self.state.affinities = Some(affinities);
            }
            Msg::AffinitiesFetchErr => {
                log::error!("Affinities fetch error");
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
                {
                if let Some(affinities) = &self.state.affinities {
                    self.view_affinities(affinities)
                } else {
                    self.view_loading()
                }
                }
        }
    }
}

impl AffinitiesPage {
    fn view_affinities(&self, affinities: &Vec<UserAffinity>) -> Html {
        html! {
            <div class="user-affinities">
            { for affinities.iter().map(|a| self.view_affinity(a)) }
            </div>
        }
    }

    fn view_affinity(&self, affinity: &UserAffinity) -> Html {
        html! {
            <div class="user-affinity">
                <div class="user">{ format!("{:?}", affinity.user) }</div>
                <div class="affinity">{ format!("{:?}", affinity.affinity) }</div>
            </div>
        }
    }

    fn view_loading(&self) -> Html {
        html! {
            <p>{ "Loading..."}</p>
        }
    }
}