"""AbsTokenizer implementation that wraps the Rust _AbsTokenizer."""

import symbolic_music._symbolic_music as _sm
_aria = getattr(_sm, 'symbolic_music.aria._aria')
_AbsTokenizer = _aria._AbsTokenizer


class AbsTokenizer(_AbsTokenizer):
    """Python wrapper for AbsTokenizer that implements export functions."""
    
    def export_pitch_aug(self, max_pitch_aug):
        """Export pitch augmentation function."""
        def pitch_aug_fn(seq, pitch_aug=None):
            return self.apply_pitch_aug(seq, max_pitch_aug, pitch_aug)
        return pitch_aug_fn
    
    def export_velocity_aug(self, max_num_aug_steps):
        """Export velocity augmentation function.""" 
        def velocity_aug_fn(seq, aug_step=None):
            return self.apply_velocity_aug(seq, max_num_aug_steps, aug_step)
        return velocity_aug_fn
    
    def export_tempo_aug(self, max_tempo_aug, mixup=False):
        """Export tempo augmentation function."""
        def tempo_aug_fn(seq, tempo_aug=None):
            return self.apply_tempo_aug(seq, max_tempo_aug, mixup, tempo_aug)
        return tempo_aug_fn