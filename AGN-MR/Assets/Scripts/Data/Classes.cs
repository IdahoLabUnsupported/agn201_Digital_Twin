using System.Collections.Generic;

namespace Data
{
    public class ReactorState
    {
        public ReactorState(float watts, float reactivity, float inv_period)
        {
            Watts = watts;
            Reactivity = reactivity;
            Inv_Period = inv_period;
        }

        public ReactorState()
        {
            Watts = 0;
            Reactivity = 0;
            Inv_Period = 0;
        }

        public float Watts { get; set; }
        public float Reactivity { get; set; }
        public float Inv_Period { get; set; }
    }

    public class PredictedState
    {
        public PredictedState(float time, float predicted, float reported, float delta)
        {
            Time = time;
            Predicted = predicted;
            Reported = reported;
            Delta = delta;
        }
        public PredictedState(string var)
        {
            Time = 0;
            Predicted = 0;
            Reported = 0;
            Delta = 0;
            Var = var;
        }

        public float Time { get; set; }
        public float Predicted { get; set; }
        public float Reported { get; set; }
        public float Delta { get; set; }
        public string Var { get; set; }
        public List<float> TimeQueryChunk { get; set; }
        public List<float> PredictedQueryChunk { get; set; }
        public List<float> ReportedQueryChunk { get; set; }
    }

    public class RodState
    {
        public RodState(float fcr_up, float fcr_engaged, float fcr_down, float ccr_up, float ccr_engaged, float ccr_down, float sr1_up, float sr1_engaged, float sr1_down, float sr2_up, float sr2_engaged, float sr2_down)
        {
            Fcr_Up = fcr_up;
            Fcr_Engaged = fcr_engaged;
            Fcr_Down = fcr_down;

            Ccr_Up = ccr_up;
            Ccr_Engaged = ccr_engaged;
            Ccr_Down = ccr_down;

            Sr1_Up = sr1_up;
            Sr1_Engaged = sr1_engaged;
            Sr1_Down = sr1_down;

            Sr2_Up = sr2_up;
            Sr2_Engaged = sr2_engaged;
            Sr2_Down = sr2_down;
        }

        public RodState()
        {
            Fcr_Up = 0;
            Fcr_Engaged = 0;
            Fcr_Down = 0;

            Ccr_Up = 0;
            Ccr_Engaged = 0;
            Ccr_Down = 0;

            Sr1_Up = 0;
            Sr1_Engaged = 0;
            Sr1_Down = 0;

            Sr2_Up = 0;
            Sr2_Engaged = 0;
            Sr2_Down = 0;
        }

        public float Fcr_Up { get; set; }
        public float Fcr_Engaged { get; set; }
        public float Fcr_Down { get; set; }

        public float Ccr_Up { get; set; }
        public float Ccr_Engaged { get; set; }
        public float Ccr_Down { get; set; }

        public float Sr1_Up { get; set; }
        public float Sr1_Engaged { get; set; }
        public float Sr1_Down { get; set; }

        public float Sr2_Up { get; set; }
        public float Sr2_Engaged { get; set; }
        public float Sr2_Down { get; set; }
    }
}
